//! Error types

use flex_error::{define_error, DisplayError, TraceError};
use prost::DecodeError;
use prost_amino::DecodeError as AminoDecodeError;
use signature::Error as SignatureError;

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
            [ TraceError<std::io::Error> ]
            | _ | { "io error" },

        Decode
            [ TraceError<DecodeError> ]
            | _ | { "malformed handshake message (protocol version mismatch?)" },

        AminoDecode
            [ TraceError<AminoDecodeError> ]
            | _ | { "malformed handshake message (protocol version mismatch?)" },

        MissingSecret
            | _ | { "missing secret: forgot to call Handshake::new?" },

        MissingKey
            | _ | { "public key missing" },

        Signature
            [ TraceError<SignatureError> ]
            | _ | { "signature error" },

        UnsupportedKey
            | _ | { "secp256k1 is not supported" },

        Aead
            [ DisplayError<aead::Error> ]
            | _ | { "aead error" },

        ShortCiphertext
            { tag_size: usize }
            | e | { format_args!("ciphertext must be at least as long as a MAC tag {}", e.tag_size) },

        SmallOutputBuffer
            | _ | { "output buffer is too small" },

    }
}
