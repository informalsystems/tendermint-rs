//! Errors raised by the builder DSL

use flex_error::define_error;
use tendermint::block::Height;
use tendermint::Hash;

use crate::components::io::IoError;
use crate::predicates::errors::VerificationError;

define_error! {
    Error {
        Io
            [ IoError ]
            | _ | { "I/O error" },

        HeightMismatch
            {
                given: Height,
                found: Height,
            }
            | e | {
                format_args!("height mismatch: given = {0}, found = {1}",
                    e.given, e.found)
            },

        HashMismatch
            {
                given: Hash,
                found: Hash,
            }
            | e | {
                format_args!("hash mismatch: given = {0}, found = {1}",
                    e.given, e.found)
            },

        InvalidLightBlock
            [ VerificationError ]
            | _ | { "invalid light block" },

        NoTrustedStateInStore
            | _ | { "no trusted state in store" },

        EmptyWitnessList
            | _ | { "empty witness list" },

    }
}
