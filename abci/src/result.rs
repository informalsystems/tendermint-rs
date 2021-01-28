//! tendermint-abci error and result handling.

use thiserror::Error;

/// Result type used throughout the crate.
pub type Result<T> = std::result::Result<T, self::Error>;

/// Errors that can be produced by tendermint-abci.
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("protobuf encoding error")]
    ProtobufEncode(#[from] prost::EncodeError),

    #[error("protobuf decoding error")]
    ProtobufDecode(#[from] prost::DecodeError),

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
