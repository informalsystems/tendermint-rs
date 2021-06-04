//! This module defines the various errors that be raised during Protobuf conversions.

use anomaly::{BoxError, Context};
use thiserror::Error;

/// An error that can be raised by the Protobuf conversions.
pub type Error = anomaly::Error<Kind>;

/// Various kinds of errors that can be raised.
#[derive(Clone, Debug, Error)]
pub enum Kind {
    /// TryFrom Prost Message failed during decoding
    #[error("error converting message type into domain type")]
    TryFromProtobuf,

    /// encoding prost Message into buffer failed
    #[error("error encoding message into buffer")]
    EncodeMessage,

    /// decoding buffer into prost Message failed
    #[error("error decoding buffer into message")]
    DecodeMessage,
}

impl Kind {
    /// Add a given source error as context for this error kind
    ///
    /// This is typically use with `map_err` as follows:
    ///
    /// ```ignore
    /// let x = self.something.do_stuff()
    ///     .map_err(|e| error::Kind::Config.context(e))?;
    /// ```
    pub fn context(self, source: impl Into<BoxError>) -> Context<Self> {
        Context::new(self, Some(source.into()))
    }
}
