use crate::prelude::*;

use bytes::Bytes;

use super::super::{params::ConsensusParams, types::ValidatorUpdate};

#[doc = include_str!("../doc/response-initchain.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct InitChain {
    /// Initial consensus-critical parameters (optional).
    pub consensus_params: Option<ConsensusParams>,
    /// Initial validator set (optional).
    ///
    /// If this list is empty, the initial validator set will be the one given in
    /// [`request::InitChain::validators`](super::super::request::InitChain::validators).
    ///
    /// If this list is nonempty, it will be the initial validator set, instead
    /// of the one given in
    /// [`request::InitChain::validators`](super::super::request::InitChain::validators).
    pub validators: Vec<ValidatorUpdate>,
    /// Initial application hash.
    pub app_hash: Bytes,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::{TryFrom, TryInto};
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<InitChain> for pb::ResponseInitChain {
    fn from(init_chain: InitChain) -> Self {
        Self {
            consensus_params: init_chain.consensus_params.map(Into::into),
            validators: init_chain.validators.into_iter().map(Into::into).collect(),
            app_hash: init_chain.app_hash,
        }
    }
}

impl TryFrom<pb::ResponseInitChain> for InitChain {
    type Error = crate::Error;

    fn try_from(init_chain: pb::ResponseInitChain) -> Result<Self, Self::Error> {
        Ok(Self {
            consensus_params: init_chain
                .consensus_params
                .map(TryInto::try_into)
                .transpose()?,
            validators: init_chain
                .validators
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            app_hash: init_chain.app_hash,
        })
    }
}

impl Protobuf<pb::ResponseInitChain> for InitChain {}
