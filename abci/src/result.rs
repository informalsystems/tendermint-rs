//! Results and errors relating to ABCI client/server operations.

use thiserror::Error;

/// Convenience type for results produced by the ABCI crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The various errors produced by the ABCI crate.
#[derive(Debug, Error)]
pub enum Error {
    #[error("protocol buffers error")]
    Protobuf(#[from] tendermint_proto::Error),

    #[error("network I/O error")]
    NetworkIo(#[from] std::io::Error),

    #[cfg(any(feature = "runtime-tokio", feature = "runtime-async-std"))]
    #[error("channel send error: {0}")]
    ChannelSend(String),

    #[cfg(feature = "runtime-tokio")]
    #[error("failed to obtain UNIX stream path")]
    CannotObtainUnixStreamPath,

    #[error("Tendermint error")]
    TendermintError(#[from] tendermint::Error),

    #[error("server stream terminated unexpectedly")]
    ServerStreamTerminated,

    #[error("sending end of channel closed unexpectedly")]
    ChannelSenderClosed,

    #[cfg(feature = "runtime-async-std")]
    #[error("failed to receive message from channel: {0}")]
    ChannelRecv(String),
}
