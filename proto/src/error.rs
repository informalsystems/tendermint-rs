//! This module defines the various errors that be raised during Protobuf conversions.

use displaydoc::Display;

/// An error that can be raised by the Protobuf conversions.
pub type Error = anyhow::Error;

/// Various kinds of errors that can be raised.
#[derive(Clone, Debug, Display)]
pub enum Kind {
    /// error converting message type into domain type
    TryFromProtobuf,

    /// error encoding message into buffer
    EncodeMessage,

    /// error decoding buffer into message
    DecodeMessage,
}

use std::fmt::Display;
impl Kind {
    pub fn context<C>(self, context: C) -> anyhow::Error
        where
            C: Display + Send + Sync + 'static,
    {
        anyhow::anyhow!(self).context(context)
    }
}