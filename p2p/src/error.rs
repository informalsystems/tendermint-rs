//! Error types

// Related to the `Error` definition below.
// TODO(soares): Update flex-error accordingly to address this.
#![allow(clippy::use_self)]

use flex_error::{define_error, DisplayOnly};
use prost::DecodeError;

define_error! {
    Error {
        Crypto
            | _ | { "cryptographic error" },

        InvalidKey
            | _ | { "invalid key" },

        LowOrderKey
            | _ | { "low-order points found (potential MitM attack!)" },

        Protocol
            | _ | { "protocol error" },

        MalformedHandshake
            | _ | { "malformed handshake message (protocol version mismatch?)" },

        Io
            [ DisplayOnly<std::io::Error> ]
            | _ | { "io error" },

        Decode
            [ DisplayOnly<DecodeError> ]
            | _ | { "malformed handshake message (protocol version mismatch?)" },

        MissingSecret
            | _ | { "missing secret: forgot to call Handshake::new?" },

        MissingKey
            | _ | { "public key missing" },

        Signature
            | _ | { "signature error" },

        UnsupportedKey
            | _ | { "secp256k1 is not supported" },

        Aead
            [ DisplayOnly<aead::Error> ]
            | _ | { "aead error" },

        ShortCiphertext
            { tag_size: usize }
            | e | { format_args!("ciphertext must be at least as long as a MAC tag {}", e.tag_size) },

        SmallOutputBuffer
            | _ | { "output buffer is too small" },

        TransportClone
            { detail: String }
            | e | { format_args!("failed to clone underlying transport: {}", e.detail) }

    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::io(e)
    }
}
