use crate::AppHash;

use crate::{consensus, prelude::*, validator};

#[doc = include_str!("../doc/response-initchain.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct InitChain {
    /// Initial consensus-critical parameters (optional).
    pub consensus_params: Option<consensus::Params>,
    /// Initial validator set (optional).
    ///
    /// If this list is empty, the initial validator set will be the one given in
    /// [`request::InitChain::validators`](super::super::request::InitChain::validators).
    ///
    /// If this list is nonempty, it will be the initial validator set, instead
    /// of the one given in
    /// [`request::InitChain::validators`](super::super::request::InitChain::validators).
    pub validators: Vec<validator::Update>,
    /// Initial application hash.
    pub app_hash: AppHash,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::InitChain;

    impl From<InitChain> for pb::abci::ResponseInitChain {
        fn from(init_chain: InitChain) -> Self {
            Self {
                consensus_params: init_chain.consensus_params.map(Into::into),
                validators: init_chain.validators.into_iter().map(Into::into).collect(),
                app_hash: init_chain.app_hash.into(),
            }
        }
    }

    impl TryFrom<pb::abci::ResponseInitChain> for InitChain {
        type Error = crate::Error;

        fn try_from(init_chain: pb::abci::ResponseInitChain) -> Result<Self, Self::Error> {
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
                app_hash: init_chain.app_hash.try_into()?,
            })
        }
    }

    impl Protobuf<pb::abci::ResponseInitChain> for InitChain {}
}
