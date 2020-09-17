use crate::Error;

/// RPC client-related result type alias.
pub type Result<T> = std::result::Result<T, Error>;
