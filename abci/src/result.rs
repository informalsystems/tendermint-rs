//! Results and errors relating to ABCI client/server operations.

use thiserror::Error;

/// Convenience type for results produced by the ABCI crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The various errors produced by the ABCI crate.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Tendermint error")]
    TendermintError(#[from] tendermint::Error),

    #[error("protocol buffers error")]
    Protobuf(#[from] tendermint_proto::Error),

    #[error("network I/O error")]
    NetworkIo(#[from] std::io::Error),

    #[error("channel send error: {0}")]
    ChannelSend(String),

    #[error("failed to receive message from channel: {0}")]
    ChannelRecv(String),

    #[error("sending end of channel closed unexpectedly")]
    ChannelSenderClosed,

    #[error("server stream terminated unexpectedly")]
    ServerStreamTerminated,
}
