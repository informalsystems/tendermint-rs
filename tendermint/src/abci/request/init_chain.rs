use crate::prelude::*;

use bytes::Bytes;
use chrono::{DateTime, Utc};

use super::super::{params::ConsensusParams, types::ValidatorUpdate};

/// Called on genesis to initialize chain state.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#initchain)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InitChain {
    /// The genesis time.
    pub time: DateTime<Utc>,
    /// The ID of the blockchain.
    pub chain_id: String,
    /// Initial consensus-critical parameters.
    pub consensus_params: ConsensusParams,
    /// Initial genesis validators, sorted by voting power.
    pub validators: Vec<ValidatorUpdate>,
    /// Serialized JSON bytes containing the initial application state.
    pub app_state_bytes: Bytes,
    /// Height of the initial block (typically `1`).
    pub initial_height: i64,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::{TryFrom, TryInto};
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<InitChain> for pb::RequestInitChain {
    fn from(init_chain: InitChain) -> Self {
        Self {
            time: Some(init_chain.time.into()),
            chain_id: init_chain.chain_id,
            consensus_params: Some(init_chain.consensus_params.into()),
            validators: init_chain.validators.into_iter().map(Into::into).collect(),
            app_state_bytes: init_chain.app_state_bytes,
            initial_height: init_chain.initial_height,
        }
    }
}

impl TryFrom<pb::RequestInitChain> for InitChain {
    type Error = crate::Error;

    fn try_from(init_chain: pb::RequestInitChain) -> Result<Self, Self::Error> {
        Ok(Self {
            time: init_chain.time.ok_or("missing genesis time")?.try_into()?,
            chain_id: init_chain.chain_id,
            consensus_params: init_chain
                .consensus_params
                .ok_or("missing consensus params")?
                .try_into()?,
            validators: init_chain
                .validators
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            app_state_bytes: init_chain.app_state_bytes,
            initial_height: init_chain.initial_height,
        })
    }
}

impl Protobuf<pb::RequestInitChain> for InitChain {}
