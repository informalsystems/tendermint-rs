//! ABCI servers.

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(feature = "non-blocking")]
pub mod non_blocking;

/// The default read buffer size for the server when reading incoming requests
/// from client connections.
pub const DEFAULT_SERVER_READ_BUF_SIZE: usize = 1024 * 1024;
