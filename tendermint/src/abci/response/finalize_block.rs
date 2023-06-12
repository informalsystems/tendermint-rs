use serde::{Deserialize, Serialize};

use crate::abci::{types::ExecTxResult, Event};
use crate::prelude::*;
use crate::{consensus, serializers, validator, AppHash};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalizeBlock {
    /// Set of block events emitted as part of executing the block
    #[serde(default)]
    pub events: Vec<Event>,
    /// The result of executing each transaction including the events
    /// the particular transction emitted. This should match the order
    /// of the transactions delivered in the block itself
    #[serde(default)]
    pub tx_results: Vec<ExecTxResult>,
    /// A list of updates to the validator set.
    /// These will reflect the validator set at current height + 2.
    pub validator_updates: Vec<validator::Update>,
    /// Updates to the consensus params, if any.
    #[serde(default)]
    pub consensus_param_updates: Option<consensus::Params>,
    /// The hash of the application's state.
    #[serde(default, with = "serializers::apphash_base64")]
    pub app_hash: AppHash,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

mod v0_38 {
    use super::FinalizeBlock;
    use tendermint_proto::v0_38::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<FinalizeBlock> for pb::ResponseFinalizeBlock {
        fn from(value: FinalizeBlock) -> Self {
            Self {
                events: value.events.into_iter().map(Into::into).collect(),
                tx_results: value.tx_results.into_iter().map(Into::into).collect(),
                validator_updates: value
                    .validator_updates
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                consensus_param_updates: value.consensus_param_updates.map(Into::into),
                app_hash: value.app_hash.into(),
            }
        }
    }

    impl TryFrom<pb::ResponseFinalizeBlock> for FinalizeBlock {
        type Error = crate::Error;

        fn try_from(message: pb::ResponseFinalizeBlock) -> Result<Self, Self::Error> {
            Ok(Self {
                events: message
                    .events
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
                tx_results: message
                    .tx_results
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
                validator_updates: message
                    .validator_updates
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
                consensus_param_updates: message
                    .consensus_param_updates
                    .map(TryInto::try_into)
                    .transpose()?,
                app_hash: message.app_hash.try_into()?,
            })
        }
    }

    impl Protobuf<pb::ResponseFinalizeBlock> for FinalizeBlock {}
}
