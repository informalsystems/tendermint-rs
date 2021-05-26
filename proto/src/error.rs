//! This module defines the various errors that be raised during Protobuf conversions.

use displaydoc::Display;

/// An error that can be raised by the Protobuf conversions.
pub type Error = anyhow::Error;


#[cfg(feature = "std")]
impl std::error::Error for Kind {}


/// Various kinds of errors that can be raised.
#[derive(Clone, Debug, Display)]
pub enum Kind {
    // TryFrom Prost Message failed during decoding
    /// error converting message type into domain type
    TryFromProtobuf,

    // encoding prost Message into buffer failed
    /// error encoding message into buffer
    EncodeMessage,

    // decoding buffer into prost Message failed
    /// error decoding buffer into message
    DecodeMessage,
}
