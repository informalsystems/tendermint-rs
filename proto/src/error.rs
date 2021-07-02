//! This module defines the various errors that be raised during Protobuf conversions.

use flex_error::{define_error, DisplayOnly};
use prost::{DecodeError, EncodeError};
use std::convert::TryFrom;
use std::fmt::Display;
use std::num::TryFromIntError;

define_error! {
    Error {
        TryFromProtobuf
            { reason: String }
            | e | {
                format!("error converting message type into domain type: {}",
                    e.reason)
            },

        EncodeMessage
            [ DisplayOnly<EncodeError> ]
            | _ | { "error encoding message into buffer" },

        DecodeMessage
            [ DisplayOnly<DecodeError> ]
            | _ | { "error decoding buffer into message" },

        ParseLength
            [ DisplayOnly<TryFromIntError> ]
            | _ | { "error parsing encoded length" },
    }
}

pub fn try_from_error<Raw, T, E>(e: E) -> Error
where
    E: Display,
    T: TryFrom<Raw, Error = E>,
{
    try_from_protobuf_error(format!("{}", e))
}
