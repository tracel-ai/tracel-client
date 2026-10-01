use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::Url;
use serde::{Deserialize, Serialize};

use super::{SessionStore, SessionStoreError, StoredAppSession};
use crate::console::credentials::{AccessToken, RefreshToken};

/// Keeps an app session in a file only its owner can read.
///
/// Writes go through a temporary file renamed over the session, so a reader
/// never sees half a session, and renewals are excluded across processes by
/// an advisory lock on a sibling `.lock` file.
#[derive(Debug, Clone)]
pub struct FileSessionStore {
    path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct SessionFile {
    access_token: String,
    access_token_expires_at: u64,
    refresh_token: String,
    refresh_token_expires_at: u64,
}

impl FileSessionStore {
    /// A store kept at `path`.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The store for the server at `base_url`, in this machine's local state
    /// directory: `$XDG_STATE_HOME/tracel` or `~/.local/state/tracel` on Linux,
    /// `~/Library/Application Support/tracel` on macOS, `%LOCALAPPDATA%\tracel`
    /// on Windows. Each server gets its own file, so signing in to one does
    /// not sign out of another.
    pub fn for_server(base_url: &Url) -> Result<Self, SessionStoreError> {
        let directory = local_state_directory()
            .ok_or_else(|| SessionStoreError::new("no local state directory on this machine"))?;
        Ok(Self::new(
            directory
                .join("tracel")
                .join("sessions")
                .join(format!("{}.json", server_key(base_url))),
        ))
    }

    /// Where the session is kept.
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn sibling(&self, extension: &str) -> PathBuf {
        let mut name = self.path.file_name().unwrap_or_default().to_os_string();
        name.push(extension);
        self.path.with_file_name(name)
    }

    fn ensure_directory(&self) -> io::Result<()> {
        let Some(directory) = self.path.parent() else {
            return Ok(());
        };
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(directory)
    }
}

impl SessionStore for FileSessionStore {
    fn load(&self) -> Result<Option<StoredAppSession>, SessionStoreError> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(failure("read", &self.path, error)),
        };
        let file: SessionFile = serde_json::from_slice(&bytes).map_err(|error| {
            SessionStoreError::new(format!("{} is not a session: {error}", self.path.display()))
        })?;
        Ok(Some(StoredAppSession {
            access_token: AccessToken::new(file.access_token),
            access_token_expires_at: from_unix_seconds(file.access_token_expires_at),
            refresh_token: RefreshToken::new(file.refresh_token),
            refresh_token_expires_at: from_unix_seconds(file.refresh_token_expires_at),
        }))
    }

    fn save(&self, session: &StoredAppSession) -> Result<(), SessionStoreError> {
        let file = SessionFile {
            access_token: session.access_token.as_str().to_string(),
            access_token_expires_at: unix_seconds(session.access_token_expires_at),
            refresh_token: session.refresh_token.as_str().to_string(),
            refresh_token_expires_at: unix_seconds(session.refresh_token_expires_at),
        };
        let bytes =
            serde_json::to_vec(&file).map_err(|error| SessionStoreError::new(error.to_string()))?;

        self.ensure_directory()
            .map_err(|error| failure("create the directory of", &self.path, error))?;
        let temporary = self.sibling(&format!(".{}.tmp", std::process::id()));
        write_owner_only(&temporary, &bytes)
            .and_then(|()| fs::rename(&temporary, &self.path))
            .map_err(|error| {
                let _ = fs::remove_file(&temporary);
                failure("write", &self.path, error)
            })
    }

    fn clear(&self) -> Result<(), SessionStoreError> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(failure("remove", &self.path, error)),
        }
    }

    fn with_exclusion(&self, critical: &mut dyn FnMut()) -> Result<(), SessionStoreError> {
        self.ensure_directory()
            .map_err(|error| failure("create the directory of", &self.path, error))?;
        let lock_path = self.sibling(".lock");
        let lock = open_owner_only(&lock_path, false)
            .map_err(|error| failure("open", &lock_path, error))?;
        lock.lock()
            .map_err(|error| failure("lock", &lock_path, error))?;
        critical();
        lock.unlock()
            .map_err(|error| failure("unlock", &lock_path, error))
    }
}

fn write_owner_only(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = open_owner_only(path, true)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn open_owner_only(path: &Path, truncate: bool) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(truncate);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

fn failure(action: &str, path: &Path, error: io::Error) -> SessionStoreError {
    SessionStoreError::new(format!("could not {action} {}: {error}", path.display()))
}

fn unix_seconds(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn from_unix_seconds(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

fn server_key(base_url: &Url) -> String {
    let host = base_url.host_str().unwrap_or("server");
    let key = match base_url.port() {
        Some(port) => format!("{host}_{port}"),
        None => host.to_string(),
    };
    key.chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => character,
            _ => '_',
        })
        .collect()
}

fn local_state_directory() -> Option<PathBuf> {
    let from = |variable: &str| {
        std::env::var_os(variable)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    };
    if cfg!(windows) {
        from("LOCALAPPDATA")
    } else if cfg!(target_os = "macos") {
        from("HOME").map(|home| home.join("Library").join("Application Support"))
    } else {
        from("XDG_STATE_HOME")
            .or_else(|| from("HOME").map(|home| home.join(".local").join("state")))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    fn scratch_store(name: &str) -> FileSessionStore {
        let directory = std::env::temp_dir().join(format!(
            "tracel-client-file-store-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        FileSessionStore::new(directory.join("nested").join("session.json"))
    }

    fn session(access_token: &str) -> StoredAppSession {
        StoredAppSession {
            access_token: AccessToken::new(access_token),
            access_token_expires_at: from_unix_seconds(1_900_000_000),
            refresh_token: RefreshToken::new(format!("refresh-of-{access_token}")),
            refresh_token_expires_at: from_unix_seconds(1_900_500_000),
        }
    }

    #[test]
    fn a_saved_session_reads_back_whole_and_clearing_forgets_it() {
        let store = scratch_store("round-trip");

        assert!(store.load().unwrap().is_none());
        store.save(&session("first")).unwrap();
        assert_eq!(store.load().unwrap(), Some(session("first")));
        store.save(&session("second")).unwrap();
        assert_eq!(store.load().unwrap(), Some(session("second")));
        store.clear().unwrap();
        store.clear().unwrap();
        assert!(store.load().unwrap().is_none());
    }

    #[cfg(unix)]
    #[test]
    fn only_the_owner_can_read_the_session_or_list_its_directory() {
        use std::os::unix::fs::PermissionsExt;
        let store = scratch_store("permissions");

        store.save(&session("first")).unwrap();

        let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(store.path()), 0o600);
        assert_eq!(mode(store.path().parent().unwrap()), 0o700);
    }

    #[test]
    fn exclusion_lets_one_holder_in_at_a_time() {
        let store = Arc::new(scratch_store("exclusion"));
        let inside = Arc::new(AtomicUsize::new(0));
        let overlaps = Arc::new(AtomicUsize::new(0));

        std::thread::scope(|scope| {
            for _ in 0..4 {
                let store = FileSessionStore::new(store.path());
                let inside = inside.clone();
                let overlaps = overlaps.clone();
                scope.spawn(move || {
                    store
                        .with_exclusion(&mut || {
                            if inside.fetch_add(1, Ordering::SeqCst) > 0 {
                                overlaps.fetch_add(1, Ordering::SeqCst);
                            }
                            std::thread::sleep(Duration::from_millis(20));
                            inside.fetch_sub(1, Ordering::SeqCst);
                        })
                        .unwrap();
                });
            }
        });

        assert_eq!(overlaps.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn each_server_gets_its_own_file() {
        let production = server_key(&Url::parse("https://console.tracel.ai/api/").unwrap());
        let local = server_key(&Url::parse("http://localhost:9001/").unwrap());

        assert_eq!(production, "console.tracel.ai");
        assert_eq!(local, "localhost_9001");
    }
}
