use flex_error::define_error;
use tendermint::{block::Height, Hash, Time};

use crate::{components::io::IoError, errors::Error as LightClientError};

define_error! {
    #[derive(Debug)]
    DetectorError {
        Io
            [ IoError ]
            |_| { "I/O error" },

        LightClient
            [ LightClientError ]
            |_| { "light client error" },

        TargetBlockLowerThanTrusted
            {
                target_height: Height,
                trusted_height: Height,
            }
            |e| {
                format_args!(
                    "target block height ({}) lower than trusted block height ({})",
                    e.target_height, e.trusted_height
                )
            },

        TrustedHashDifferentFromSourceFirstBlock
            {
                expected_hash: Hash,
                got_hash: Hash,
            }
            |e| {
                format_args!(
                    "trusted block is different to the source's first block. Expected hash: {}, got: {}",
                    e.expected_hash, e.got_hash
                )
            },

        TraceBlockAfterTargetBlock
            {
                trace_time: Time,
                target_time: Time,
            }
            |e| {
                format_args!(
                    "trace block ({}) is after target block ({})",
                    e.trace_time, e.target_time
                )
            },

        NoDivergence
            |_| { "expected divergence between conflicting headers but none found" },
    }
}
