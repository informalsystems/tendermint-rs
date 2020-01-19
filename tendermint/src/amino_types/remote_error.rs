use prost_amino_derive::Message;

#[derive(Clone, PartialEq, Message)]
pub struct RemoteError {
    #[prost_amino(sint32, tag = "1")]
    pub code: i32,
    #[prost_amino(string, tag = "2")]
    pub description: String,
}

/// Error codes for remote signer failures
// TODO(tarcieri): add these to Tendermint. See corresponding TODO here:
// <https://github.com/tendermint/tendermint/blob/master/privval/errors.go>
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum RemoteErrorCode {
    /// Generic error code useful when the others don't apply
    RemoteSignerError = 1,

    /// Double signing detected
    DoubleSignError = 2,
}

impl RemoteError {
    /// Create a new double signing error with the given message
    pub fn double_sign(height: i64) -> Self {
        RemoteError {
            code: RemoteErrorCode::DoubleSignError as i32,
            description: format!("double signing requested at height: {}", height),
        }
    }
}
