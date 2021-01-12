//! Results and errors relating to ABCI client/server operations.

use thiserror::Error;

/// Convenience type for results produced by the ABCI crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The various errors produced by the ABCI crate.
#[derive(Debug, Error)]
pub enum Error {
    #[error("protocol buffers error")]
    Protobuf(#[from] tendermint_proto::Error),

    #[cfg(feature = "with-tokio")]
    #[error("network I/O error")]
    TokioIo(#[from] tokio::io::Error),

    #[cfg(feature = "with-tokio")]
    #[error("channel send error: {0}")]
    TokioChannelSend(String),

    #[cfg(feature = "with-tokio")]
    #[error("failed to obtain UNIX stream path")]
    CannotObtainUnixStreamPath,

    #[error("Tendermint error")]
    TendermintError(#[from] tendermint::Error),

    #[error("server stream terminated unexpectedly")]
    ServerStreamTerminated,
}
