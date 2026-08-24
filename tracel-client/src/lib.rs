#[cfg(not(any(feature = "console", feature = "station")))]
compile_error!("tracel-client requires at least one of the `console` or `station` features");

pub mod error;
pub mod websocket;

mod transport;

#[cfg(feature = "console")]
pub mod console;
#[cfg(feature = "station")]
pub mod station;

pub use error::{ApiErrorCode, ClientError};
pub use websocket::WebSocketClient;
