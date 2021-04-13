//! Consensus-relevant parameters that can be adjusted by the ABCI app.

use crate::prelude::*;

use chrono::Duration;
use core::convert::{TryFrom, TryInto};

/// All consensus-relevant parameters that can be adjusted by the ABCI app.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#consensusparams)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ConsensusParams {
    /// Parameters limiting the size of a block and time between consecutive blocks.
    pub block: BlockParams,
    /// Parameters limiting the validity of evidence of byzantine behaviour.
    pub evidence: EvidenceParams,
    /// Parameters limiting the types of public keys validators can use.
    pub validator: ValidatorParams,
    /// The ABCI application version.
    pub version: VersionParams,
}
/// BlockParams contains limits on the block size.
///
/// [Tendermint documentation](https://docs.tendermint.com/master/spec/core/data_structures.html#blockparams)
///
/// XXX(hdevalence): this isn't ABCI-specific, should it live here?
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BlockParams {
    /// Max block size, in bytes.
    pub max_bytes: i64,
    /// Max sum of `gas_wanted` in a proposed block.
    ///
    /// Note: blocks that violate this may be committed if there are Byzantine
    /// proposers. It's the application's responsibility to handle this when
    /// processing a block.
    pub max_gas: i64,
}

/// EvidenceParams determine how we handle evidence of malfeasance.
///
/// [Tendermint documentation](https://docs.tendermint.com/master/spec/core/data_structures.html#evidenceparams)
///
/// XXX(hdevalence): this isn't ABCI-specific, should it live here?
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EvidenceParams {
    /// Max age of evidence, in blocks.
    pub max_age_num_blocks: i64,
    /// Max age of evidence, in time.
    ///
    /// It should correspond with an app's "unbonding period" or other similar
    /// mechanism for handling [Nothing-At-Stake attacks][nas].
    ///
    /// [nas]: https://github.com/ethereum/wiki/wiki/Proof-of-Stake-FAQ#what-is-the-nothing-at-stake-problem-and-how-can-it-be-fixed
    pub max_age_duration: Duration,
    /// This sets the maximum size of total evidence in bytes that can be
    /// committed in a single block, and should fall comfortably under the max
    /// block bytes. The default is 1048576 or 1MB.
    pub max_bytes: i64,
}

/// ValidatorParams restrict the public key types validators can use.
///
/// [Tendermint documentation](https://docs.tendermint.com/master/spec/core/data_structures.html#validatorparams)
///
/// XXX(hdevalence): this isn't ABCI-specific, should it live here?
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ValidatorParams {
    /// List of accepted public key types.
    pub pub_key_types: Vec<String>,
}

/// (No description)
///
/// [Tendermint documentation](https://docs.tendermint.com/master/spec/core/data_structures.html#versionparams)
///
/// XXX(hdevalence): this isn't ABCI-specific, should it live here?
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VersionParams {
    /// The ABCI application version.
    pub app_version: u64,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use tendermint_proto::types as pb;
use tendermint_proto::Protobuf;

impl From<ConsensusParams> for pb::ConsensusParams {
    fn from(params: ConsensusParams) -> Self {
        Self {
            block: Some(params.block.into()),
            evidence: Some(params.evidence.into()),
            validator: Some(params.validator.into()),
            version: Some(params.version.into()),
        }
    }
}

impl TryFrom<pb::ConsensusParams> for ConsensusParams {
    type Error = crate::Error;

    fn try_from(params: pb::ConsensusParams) -> Result<Self, Self::Error> {
        Ok(Self {
            block: params.block.ok_or("missing block params")?.try_into()?,
            evidence: params
                .evidence
                .ok_or("missing evidence params")?
                .try_into()?,
            validator: params
                .validator
                .ok_or("missing validator params")?
                .try_into()?,
            version: params.version.ok_or("missing version params")?.try_into()?,
        })
    }
}

impl Protobuf<pb::ConsensusParams> for ConsensusParams {}

impl From<BlockParams> for pb::BlockParams {
    fn from(params: BlockParams) -> Self {
        Self {
            max_bytes: params.max_bytes,
            max_gas: params.max_gas,
        }
    }
}

impl TryFrom<pb::BlockParams> for BlockParams {
    type Error = crate::Error;

    fn try_from(params: pb::BlockParams) -> Result<Self, Self::Error> {
        if params.max_bytes == 0 {
            Err("BlockParams::max_bytes must be greater than 0")?
        }
        if params.max_gas < -1 {
            Err("BlockParams::max_gas must be greater than or equal to -1")?
        }

        Ok(Self {
            max_bytes: params.max_bytes,
            max_gas: params.max_gas,
        })
    }
}

impl Protobuf<pb::BlockParams> for BlockParams {}

impl From<EvidenceParams> for pb::EvidenceParams {
    fn from(params: EvidenceParams) -> Self {
        Self {
            max_age_num_blocks: params.max_age_num_blocks,
            max_age_duration: Some(params.max_age_duration.into()),
            max_bytes: params.max_bytes,
        }
    }
}

impl TryFrom<pb::EvidenceParams> for EvidenceParams {
    type Error = crate::Error;

    fn try_from(params: pb::EvidenceParams) -> Result<Self, Self::Error> {
        Ok(Self {
            max_age_num_blocks: params.max_age_num_blocks,
            max_age_duration: params
                .max_age_duration
                .ok_or("missing max age duration")?
                .into(),
            max_bytes: params.max_bytes,
        })
    }
}

impl Protobuf<pb::EvidenceParams> for EvidenceParams {}

impl From<ValidatorParams> for pb::ValidatorParams {
    fn from(params: ValidatorParams) -> Self {
        Self {
            pub_key_types: params.pub_key_types,
        }
    }
}

impl TryFrom<pb::ValidatorParams> for ValidatorParams {
    type Error = crate::Error;

    fn try_from(params: pb::ValidatorParams) -> Result<Self, Self::Error> {
        Ok(Self {
            pub_key_types: params.pub_key_types,
        })
    }
}

impl Protobuf<pb::ValidatorParams> for ValidatorParams {}

impl From<VersionParams> for pb::VersionParams {
    fn from(params: VersionParams) -> Self {
        Self {
            app_version: params.app_version,
        }
    }
}

impl TryFrom<pb::VersionParams> for VersionParams {
    type Error = crate::Error;

    fn try_from(params: pb::VersionParams) -> Result<Self, Self::Error> {
        Ok(Self {
            app_version: params.app_version,
        })
    }
}

impl Protobuf<pb::VersionParams> for VersionParams {}
