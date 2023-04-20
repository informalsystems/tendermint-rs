use tendermint::{block::Height, Hash, Time};
use tendermint_light_client::components::io::IoError;
use tendermint_light_client::errors::Error as LightClientError;
use tendermint_light_client::verifier::types::LightBlock;

use crate::conflict::GatheredEvidence;

flex_error::define_error! {
    /// Error type for the light client detector. See [`ErrorDetail`] for all the possible error variants.
    ///
    /// All the possible error variants.
    #[derive(Debug)]
    Error {
        Io
            [ IoError ]
            |_| { "I/O error" },

        LightClient
            [ LightClientError ]
            |_| { "light client error" },

        NoDivergence
            |_| { "expected divergence between conflicting headers but none found" },

        Divergence
            {
                evidence: GatheredEvidence,
                challenging_block: LightBlock,
            }
            |e| { format_args!("divergence detected, found evidence: {:#?}", e.evidence) },

        NoWitnesses
            |_| { "no witnesses provided" },

        BadWitness
            |_| { "bad witness" },

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

        TraceTooShort
            {
                trace: Vec<LightBlock>,
            }
            |e| {
                format_args!(
                    "trace is too short. Expected at least 2 blocks, got {} block",
                    e.trace.len()
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

        FailedHeaderCrossReferencing
            |_| { format_args!("failed to cross-reference header with witness") },
    }
}
