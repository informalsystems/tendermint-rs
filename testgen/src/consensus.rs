use tendermint::consensus::params::VersionParams;
use tendermint::{block, consensus, evidence, public_key::Algorithm};

/// Default consensus params modeled after Go code; but it's not clear how to go to a valid hash
/// from here
pub fn default_consensus_params() -> consensus::Params {
    consensus::Params {
        block: block::Size {
            max_bytes: 22020096,
            max_gas: -1, // Tendetmint-go also has TimeIotaMs: 1000, // 1s
        },
        evidence: evidence::Params {
            max_age_num_blocks: 100000,
            max_age_duration: evidence::Duration(std::time::Duration::new(48 * 3600, 0)),
            max_bytes: 1048576,
        },
        validator: consensus::params::ValidatorParams {
            pub_key_types: vec![Algorithm::Ed25519],
        },
        version: Some(VersionParams::default()),
    }
}
