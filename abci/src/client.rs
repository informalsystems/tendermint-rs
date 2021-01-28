//! ABCI clients for interacting with ABCI servers.

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(feature = "non-blocking")]
pub mod non_blocking;

/// The default size of the ABCI client's read buffer when reading responses
/// from the server.
pub const DEFAULT_CLIENT_READ_BUF_SIZE: usize = 1024;
