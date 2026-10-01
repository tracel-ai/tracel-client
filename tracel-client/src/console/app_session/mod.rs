//! An app session the client renews by itself.
//!
//! The device flow signs an app in with an access token that lasts an hour
//! and a refresh token that renews it until the app session ends, seven days
//! after sign-in. An [`AppSession`] keeps both in a [`SessionStore`] and
//! renews the access token when it is about to expire or when the server
//! refuses it, so a [`Client`](crate::console::Client) connected with it keeps
//! working for the whole week.
//!
//! Several processes can share one store, such as the `tracel` CLI and a
//! training run started from it. A renewal spends the refresh token, so it
//! runs under the store's exclusion: whoever renews second finds the
//! session the first one saved and adopts it instead of spending a token that
//! is already spent.
//!
//! # Examples
//!
//! ```no_run
//! use std::sync::Arc;
//!
//! use tracel_client::console::app_session::{AppSession, FileSessionStore};
//! use tracel_client::console::auth::DeviceAuthClient;
//! use tracel_client::console::{Client, Env, TracelCredentials};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let env = Env::Production;
//! let store = FileSessionStore::for_server(&env.get_url())?;
//! let app_session = AppSession::new(Arc::new(store), DeviceAuthClient::new(env.clone(), "tracel-cli"));
//!
//! let issued = DeviceAuthClient::new(env.clone(), "tracel-cli").authorize(|auth| {
//!     println!("Open {} and enter {}", auth.verification_uri, auth.user_code);
//! })?;
//! app_session.sign_in(issued)?;
//!
//! let client = Client::connect(env, &TracelCredentials::app_session(app_session))?;
//! # Ok(())
//! # }
//! ```

mod file_store;

use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use thiserror::Error;

use crate::console::auth::{DeviceAuthClient, DeviceFlowError, IssuedAppSession};
use crate::console::credentials::{AccessToken, RefreshToken};
use crate::error::ClientError;

pub use file_store::FileSessionStore;

/// How long before its expiry an access token is renewed, so a request never
/// leaves with a token that expires on the way.
const RENEWAL_MARGIN: Duration = Duration::from_secs(60);

/// An app session as a [`SessionStore`] keeps it.
#[derive(Clone, PartialEq, Eq)]
pub struct StoredAppSession {
    pub access_token: AccessToken,
    pub access_token_expires_at: SystemTime,
    pub refresh_token: RefreshToken,
    /// When the app session ends and the user has to sign in again.
    pub refresh_token_expires_at: SystemTime,
}

impl StoredAppSession {
    /// The session the token endpoint granted at `issued_at`.
    pub fn issued(issued: IssuedAppSession, issued_at: SystemTime) -> Self {
        Self {
            access_token: issued.access_token,
            access_token_expires_at: issued_at + issued.access_token_expires_in,
            refresh_token: issued.refresh_token,
            refresh_token_expires_at: issued_at + issued.refresh_token_expires_in,
        }
    }

    /// Whether the access token outlives `now` by the renewal margin.
    pub fn access_token_is_fresh_at(&self, now: SystemTime) -> bool {
        now + RENEWAL_MARGIN < self.access_token_expires_at
    }

    /// Whether the app session has ended at `now`.
    pub fn has_ended_at(&self, now: SystemTime) -> bool {
        now >= self.refresh_token_expires_at
    }
}

/// Redacts the tokens.
impl Debug for StoredAppSession {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredAppSession")
            .field("access_token", &self.access_token)
            .field("access_token_expires_at", &self.access_token_expires_at)
            .field("refresh_token", &self.refresh_token)
            .field("refresh_token_expires_at", &self.refresh_token_expires_at)
            .finish()
    }
}

/// Where an [`AppSession`] keeps its tokens.
///
/// `with_exclusion` must exclude every other user of the same store, in this
/// process and in others, for as long as `critical` runs: the renewal it
/// guards spends a refresh token that only one of them may spend. A store that
/// cannot exclude must fail rather than run `critical` unguarded.
pub trait SessionStore: Send + Sync {
    /// The stored session, or `None` when nobody is signed in.
    fn load(&self) -> Result<Option<StoredAppSession>, SessionStoreError>;

    /// Replaces the stored session.
    fn save(&self, session: &StoredAppSession) -> Result<(), SessionStoreError>;

    /// Forgets the stored session.
    fn clear(&self) -> Result<(), SessionStoreError>;

    /// Runs `critical` while holding the store's exclusion.
    fn with_exclusion(&self, critical: &mut dyn FnMut()) -> Result<(), SessionStoreError>;
}

/// A [`SessionStore`] failed to read, write or lock what it keeps.
#[derive(Debug, Error)]
#[error("The app session store failed: {0}")]
pub struct SessionStoreError(String);

impl SessionStoreError {
    pub fn new(reason: impl Into<String>) -> Self {
        Self(reason.into())
    }
}

impl From<SessionStoreError> for ClientError {
    fn from(error: SessionStoreError) -> Self {
        ClientError::SessionStore(error.0)
    }
}

/// Spends a refresh token for a new pair.
///
/// [`DeviceAuthClient`] in production; a fake in tests.
trait Renewer: Send + Sync {
    fn renew(&self, refresh_token: &RefreshToken) -> Result<IssuedAppSession, DeviceFlowError>;

    fn revoke(&self, refresh_token: &RefreshToken) -> Result<(), DeviceFlowError>;
}

impl Renewer for DeviceAuthClient {
    fn renew(&self, refresh_token: &RefreshToken) -> Result<IssuedAppSession, DeviceFlowError> {
        self.refresh(refresh_token)
    }

    fn revoke(&self, refresh_token: &RefreshToken) -> Result<(), DeviceFlowError> {
        DeviceAuthClient::revoke(self, refresh_token.as_str())
    }
}

/// An app session the client renews through its [`SessionStore`].
///
/// Clones share the session: a renewal by one is seen by all.
#[derive(Clone)]
pub struct AppSession {
    inner: Arc<AppSessionInner>,
}

struct AppSessionInner {
    store: Arc<dyn SessionStore>,
    renewer: Box<dyn Renewer>,
    current: Mutex<Option<StoredAppSession>>,
}

impl AppSession {
    /// An app session kept in `store` and renewed through `device_auth`,
    /// which must name the same client the session was signed in with.
    pub fn new(store: Arc<dyn SessionStore>, device_auth: DeviceAuthClient) -> Self {
        Self::renewed_by(store, Box::new(device_auth))
    }

    fn renewed_by(store: Arc<dyn SessionStore>, renewer: Box<dyn Renewer>) -> Self {
        Self {
            inner: Arc::new(AppSessionInner {
                store,
                renewer,
                current: Mutex::new(None),
            }),
        }
    }

    /// Keeps what a device authorization just granted as the stored session.
    pub fn sign_in(&self, issued: IssuedAppSession) -> Result<(), ClientError> {
        let session = StoredAppSession::issued(issued, SystemTime::now());
        let mut current = self.current();
        self.inner.store.save(&session)?;
        *current = Some(session);
        Ok(())
    }

    /// The stored session, without renewing anything.
    pub fn stored(&self) -> Result<Option<StoredAppSession>, ClientError> {
        Ok(self.inner.store.load()?)
    }

    /// Signs the app out on the server, then forgets the stored session.
    ///
    /// Signing out with nobody signed in does nothing. If the server cannot be
    /// reached, the stored session is kept so signing out can be retried.
    pub fn sign_out(&self) -> Result<(), ClientError> {
        let mut current = self.current();
        let mut outcome = Ok(());
        self.inner.store.with_exclusion(&mut || {
            outcome = self.sign_out_holding_the_store();
        })?;
        *current = None;
        outcome
    }

    fn sign_out_holding_the_store(&self) -> Result<(), ClientError> {
        if let Some(stored) = self.inner.store.load()? {
            self.inner
                .renewer
                .revoke(&stored.refresh_token)
                .map_err(flow_failure)?;
        }
        Ok(self.inner.store.clear()?)
    }

    /// An access token good for the next request, renewed first when it
    /// would expire within a minute.
    ///
    /// For handing the session to a program that speaks to the server
    /// itself, such as a script given `tracel auth token`. A
    /// [`Client`](crate::console::Client) connected with this session asks
    /// for it on its own.
    pub fn access_token(&self) -> Result<AccessToken, ClientError> {
        let mut current = self.current();
        let session = match current.clone() {
            Some(session) => session,
            None => self
                .inner
                .store
                .load()?
                .ok_or(ClientError::AppSessionEnded)?,
        };
        if session.access_token_is_fresh_at(SystemTime::now()) {
            let access_token = session.access_token.clone();
            *current = Some(session);
            return Ok(access_token);
        }

        let renewed = self.renew(&session.access_token)?;
        let access_token = renewed.access_token.clone();
        *current = Some(renewed);
        Ok(access_token)
    }

    /// An access token to replace `rejected`, which the server refused.
    ///
    /// Another thread or process may have renewed it already; otherwise the
    /// refresh token is spent now.
    pub(crate) fn access_token_after_rejection(
        &self,
        rejected: &AccessToken,
    ) -> Result<AccessToken, ClientError> {
        let mut current = self.current();
        if let Some(session) = current.as_ref()
            && &session.access_token != rejected
            && session.access_token_is_fresh_at(SystemTime::now())
        {
            return Ok(session.access_token.clone());
        }

        let renewed = self.renew(rejected)?;
        let access_token = renewed.access_token.clone();
        *current = Some(renewed);
        Ok(access_token)
    }

    fn renew(&self, seen: &AccessToken) -> Result<StoredAppSession, ClientError> {
        let mut outcome = Err(ClientError::AppSessionEnded);
        self.inner.store.with_exclusion(&mut || {
            outcome = self.renew_holding_the_store(seen);
        })?;
        outcome
    }

    fn renew_holding_the_store(&self, seen: &AccessToken) -> Result<StoredAppSession, ClientError> {
        let now = SystemTime::now();
        let stored = self
            .inner
            .store
            .load()?
            .ok_or(ClientError::AppSessionEnded)?;
        if &stored.access_token != seen && stored.access_token_is_fresh_at(now) {
            return Ok(stored);
        }
        if stored.has_ended_at(now) {
            self.inner.store.clear()?;
            return Err(ClientError::AppSessionEnded);
        }

        match self.inner.renewer.renew(&stored.refresh_token) {
            Ok(issued) => {
                let renewed = StoredAppSession::issued(issued, now);
                self.inner.store.save(&renewed)?;
                Ok(renewed)
            }
            Err(DeviceFlowError::InvalidGrant) => {
                self.inner.store.clear()?;
                Err(ClientError::AppSessionEnded)
            }
            Err(error) => Err(flow_failure(error)),
        }
    }

    fn current(&self) -> std::sync::MutexGuard<'_, Option<StoredAppSession>> {
        self.inner
            .current
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn flow_failure(error: DeviceFlowError) -> ClientError {
    match error {
        DeviceFlowError::Client(error) => error,
        error => ClientError::UnknownError(error.to_string()),
    }
}

/// Redacts the session.
impl Debug for AppSession {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("AppSession([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[derive(Default)]
    struct MemoryStore {
        session: Mutex<Option<StoredAppSession>>,
        exclusion: Mutex<()>,
    }

    impl SessionStore for MemoryStore {
        fn load(&self) -> Result<Option<StoredAppSession>, SessionStoreError> {
            Ok(self.session.lock().unwrap().clone())
        }

        fn save(&self, session: &StoredAppSession) -> Result<(), SessionStoreError> {
            *self.session.lock().unwrap() = Some(session.clone());
            Ok(())
        }

        fn clear(&self) -> Result<(), SessionStoreError> {
            *self.session.lock().unwrap() = None;
            Ok(())
        }

        fn with_exclusion(&self, critical: &mut dyn FnMut()) -> Result<(), SessionStoreError> {
            let _held = self.exclusion.lock().unwrap();
            critical();
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct CountingRenewer {
        renewals: Arc<AtomicUsize>,
        revocations: Arc<AtomicUsize>,
        refuses: bool,
    }

    impl Renewer for CountingRenewer {
        fn renew(
            &self,
            _refresh_token: &RefreshToken,
        ) -> Result<IssuedAppSession, DeviceFlowError> {
            let renewal = self.renewals.fetch_add(1, Ordering::SeqCst) + 1;
            if self.refuses {
                return Err(DeviceFlowError::InvalidGrant);
            }
            Ok(issued(
                &format!("renewed-{renewal}"),
                Duration::from_secs(3600),
            ))
        }

        fn revoke(&self, _refresh_token: &RefreshToken) -> Result<(), DeviceFlowError> {
            self.revocations.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn issued(access_token: &str, access_token_expires_in: Duration) -> IssuedAppSession {
        IssuedAppSession {
            access_token: AccessToken::new(access_token),
            access_token_expires_in,
            refresh_token: RefreshToken::new(format!("refresh-of-{access_token}")),
            refresh_token_expires_in: Duration::from_secs(7 * 24 * 3600),
        }
    }

    fn signed_in(
        store: &Arc<MemoryStore>,
        renewer: &CountingRenewer,
        access_token_expires_in: Duration,
    ) -> AppSession {
        let session = AppSession::renewed_by(store.clone(), Box::new(renewer.clone()));
        session
            .sign_in(issued("first", access_token_expires_in))
            .unwrap();
        session
    }

    #[test]
    fn a_fresh_access_token_is_used_without_renewing() {
        let store = Arc::new(MemoryStore::default());
        let renewer = CountingRenewer::default();
        let session = signed_in(&store, &renewer, Duration::from_secs(3600));

        let access_token = session.access_token().unwrap();

        assert_eq!(access_token, AccessToken::new("first"));
        assert_eq!(renewer.renewals.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn an_access_token_about_to_expire_is_renewed_and_the_renewal_is_stored_before_use() {
        let store = Arc::new(MemoryStore::default());
        let renewer = CountingRenewer::default();
        let session = signed_in(&store, &renewer, Duration::from_secs(30));

        let access_token = session.access_token().unwrap();

        assert_eq!(access_token, AccessToken::new("renewed-1"));
        assert_eq!(
            store.load().unwrap().unwrap().access_token,
            AccessToken::new("renewed-1")
        );
    }

    #[test]
    fn a_session_another_process_already_renewed_is_adopted_without_spending_its_token() {
        let store = Arc::new(MemoryStore::default());
        let renewer = CountingRenewer::default();
        let ours = signed_in(&store, &renewer, Duration::from_secs(30));
        let theirs = AppSession::renewed_by(store.clone(), Box::new(renewer.clone()));
        theirs.access_token().unwrap();

        let access_token = ours.access_token().unwrap();

        assert_eq!(access_token, AccessToken::new("renewed-1"));
        assert_eq!(renewer.renewals.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn threads_sharing_an_expiring_session_renew_it_once() {
        let store = Arc::new(MemoryStore::default());
        let renewer = CountingRenewer::default();
        signed_in(&store, &renewer, Duration::from_secs(30));
        let sessions: Vec<_> = (0..8)
            .map(|_| AppSession::renewed_by(store.clone(), Box::new(renewer.clone())))
            .collect();

        let tokens: Vec<_> = std::thread::scope(|scope| {
            let handles: Vec<_> = sessions
                .iter()
                .map(|session| scope.spawn(|| session.access_token().unwrap()))
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });

        assert_eq!(renewer.renewals.load(Ordering::SeqCst), 1);
        assert!(
            tokens
                .iter()
                .all(|token| *token == AccessToken::new("renewed-1"))
        );
    }

    #[test]
    fn a_rejected_access_token_is_renewed_even_while_it_looks_fresh() {
        let store = Arc::new(MemoryStore::default());
        let renewer = CountingRenewer::default();
        let session = signed_in(&store, &renewer, Duration::from_secs(3600));

        let access_token = session
            .access_token_after_rejection(&AccessToken::new("first"))
            .unwrap();

        assert_eq!(access_token, AccessToken::new("renewed-1"));
    }

    #[test]
    fn a_refused_refresh_token_ends_the_session_and_forgets_it() {
        let store = Arc::new(MemoryStore::default());
        let renewer = CountingRenewer {
            refuses: true,
            ..CountingRenewer::default()
        };
        let session = signed_in(&store, &renewer, Duration::from_secs(30));

        let result = session.access_token();

        assert!(matches!(result, Err(ClientError::AppSessionEnded)));
        assert!(store.load().unwrap().is_none());
    }

    #[test]
    fn nobody_signed_in_reads_as_an_ended_session() {
        let session = AppSession::renewed_by(
            Arc::new(MemoryStore::default()),
            Box::new(CountingRenewer::default()),
        );

        assert!(matches!(
            session.access_token(),
            Err(ClientError::AppSessionEnded)
        ));
    }

    #[test]
    fn signing_out_revokes_on_the_server_then_forgets_the_session() {
        let store = Arc::new(MemoryStore::default());
        let renewer = CountingRenewer::default();
        let session = signed_in(&store, &renewer, Duration::from_secs(3600));

        session.sign_out().unwrap();

        assert_eq!(renewer.revocations.load(Ordering::SeqCst), 1);
        assert!(store.load().unwrap().is_none());
        assert!(matches!(
            session.access_token(),
            Err(ClientError::AppSessionEnded)
        ));
    }

    #[test]
    fn a_stored_session_does_not_print_its_tokens() {
        let stored =
            StoredAppSession::issued(issued("secret", Duration::from_secs(1)), SystemTime::now());

        let printed = format!("{stored:?}");

        assert!(!printed.contains("secret"));
    }
}
