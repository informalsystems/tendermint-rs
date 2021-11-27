//! Errors relating to the RPC probe's operations.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("an internal error occurred: {0}")]
    Internal(String),

    #[error("WebSocket connection error: {0}")]
    WebSocket(String),

    #[error("timed out: {0}")]
    Timeout(String),

    #[error("malformed RPC response: {0}")]
    MalformedResponse(String),

    #[error("\"{0}\" request failed with response: {1}")]
    Failed(String, serde_json::Value),

    #[error("invalid parameter value: {0}")]
    InvalidParamValue(String),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("unexpected success response")]
    UnexpectedSuccessResponse,

    #[error("unexpected error response: {0}")]
    UnexpectedErrorResponse(String),
}

impl From<async_tungstenite::tungstenite::Error> for Error {
    fn from(e: async_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocket(e.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for Error {
    fn from(e: tokio::time::error::Elapsed) -> Self {
        Self::Timeout(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::MalformedResponse(e.to_string())
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(e: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::Internal(format!("failed to send to channel: {}", e))
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(e: tokio::task::JoinError) -> Self {
        Self::Internal(format!(
            "failed while waiting for async task to join: {}",
            e
        ))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}
