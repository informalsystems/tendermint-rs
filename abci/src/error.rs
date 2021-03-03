//! tendermint-abci errors

use thiserror::Error;

/// Errors that can be produced by tendermint-abci.
#[derive(Debug, Error)]
pub enum Error {
    #[error("server connection terminated")]
    ServerConnectionTerminated,

    #[error("malformed server response")]
    MalformedServerResponse,

    #[error("unexpected server response type: expected {0}, but got {1:?}")]
    UnexpectedServerResponseType(String, tendermint_proto::abci::response::Value),

    #[error("channel send error: {0}")]
    ChannelSend(String),

    #[error("channel receive error: {0}")]
    ChannelRecv(String),
}
