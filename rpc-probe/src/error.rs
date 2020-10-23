//! Errors relating to the RPC probe's operations.

use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio::task::JoinError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("an internal error occurred: {0}")]
    InternalError(String),

    #[error("WebSocket connection error: {0}")]
    WebSocketError(String),

    #[error("timed out: {0}")]
    Timeout(String),

    #[error("malformed RPC response: {0}")]
    MalformedResponse(String),

    #[error("{0} request failed: {1}")]
    Failed(String, String),
}

impl From<async_tungstenite::tungstenite::Error> for Error {
    fn from(e: async_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocketError(e.to_string())
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
        Self::InternalError(format!("failed to send to channel: {}", e))
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(e: tokio::task::JoinError) -> Self {
        Self::InternalError(format!(
            "failed while waiting for async task to join: {}",
            e
        ))
    }
}
