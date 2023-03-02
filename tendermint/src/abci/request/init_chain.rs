use bytes::Bytes;

use crate::{block, consensus, prelude::*, validator, Time};

/// Called on genesis to initialize chain state.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#initchain)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InitChain {
    /// The genesis time.
    pub time: Time,
    /// The ID of the blockchain.
    pub chain_id: String,
    /// Initial consensus-critical parameters.
    pub consensus_params: consensus::Params,
    /// Initial genesis validators, sorted by voting power.
    pub validators: Vec<validator::Update>,
    /// Serialized JSON bytes containing the initial application state.
    pub app_state_bytes: Bytes,
    /// Height of the initial block (typically `1`).
    pub initial_height: block::Height,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::InitChain;
    use crate::Error;

    impl From<InitChain> for pb::abci::RequestInitChain {
        fn from(init_chain: InitChain) -> Self {
            Self {
                time: Some(init_chain.time.into()),
                chain_id: init_chain.chain_id,
                consensus_params: Some(init_chain.consensus_params.into()),
                validators: init_chain.validators.into_iter().map(Into::into).collect(),
                app_state_bytes: init_chain.app_state_bytes,
                initial_height: init_chain.initial_height.into(),
            }
        }
    }

    impl TryFrom<pb::abci::RequestInitChain> for InitChain {
        type Error = Error;

        fn try_from(init_chain: pb::abci::RequestInitChain) -> Result<Self, Self::Error> {
            Ok(Self {
                time: init_chain
                    .time
                    .ok_or_else(Error::missing_genesis_time)?
                    .try_into()?,
                chain_id: init_chain.chain_id,
                consensus_params: init_chain
                    .consensus_params
                    .ok_or_else(Error::missing_consensus_params)?
                    .try_into()?,
                validators: init_chain
                    .validators
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
                app_state_bytes: init_chain.app_state_bytes,
                initial_height: init_chain.initial_height.try_into()?,
            })
        }
    }

    impl Protobuf<pb::abci::RequestInitChain> for InitChain {}
}
