use tendermint_light_client_verifier::types::LightBlock;

use crate::misbehavior::conflict::GatheredEvidence;
use crate::misbehavior::error::DetectorError;

flex_error::define_error! {
    #[derive(Debug)]
    Error {
        NoWitnesses
            |_| { format_args!("No witnesses provided") },

        BadWitness
        |_| { format_args!("Bad witness") },

        Divergence
            {
                evidence: GatheredEvidence,
                challenging_block: LightBlock,
            }
            |e| { format_args!("Divergence detected, found evidence: {:#?}", e.evidence) },

        FailedHeaderCrossReferencing
            |_| { format_args!("Failed to cross-reference header with witness") },

        TraceTooShort
            { trace: Vec<LightBlock> }
            |e| { format_args!("Trace too short, length: {}", e.trace.len()) },

        DetectorError
            [ DetectorError ]
            |_| { format_args!("Detector error") },

        Other
            [ crate::errors::Error ]
            |_| { "Other error" },
    }
}
