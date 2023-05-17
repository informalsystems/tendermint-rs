//! `/block_results` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};
use tendermint::{abci, block, consensus, serializers, validator, AppHash};

use crate::dialect::{self, Dialect};
use crate::prelude::*;
use crate::request::RequestMessage;

/// Get ABCI results at a given height.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Height of the block to request.
    ///
    /// If no height is provided, it will fetch results for the latest block.
    pub height: Option<block::Height>,
}

impl Request {
    /// Create a new request for information about a particular block
    pub fn new(height: block::Height) -> Self {
        Self {
            height: Some(height),
        }
    }
}

impl RequestMessage for Request {
    fn method(&self) -> crate::Method {
        crate::Method::BlockResults
    }
}

impl crate::Request<dialect::v0_34::Dialect> for Request {
    type Response = self::v0_34::DialectResponse;
}

impl crate::Request<dialect::v0_37::Dialect> for Request {
    type Response = self::v0_37::DialectResponse;
}

impl<S: Dialect> crate::SimpleRequest<S> for Request
where
    Self: crate::Request<S>,
    Response: From<Self::Response>,
{
    type Output = Response;
}

/// ABCI result response.
#[derive(Clone, Debug, Serialize)]
pub struct Response {
    /// Block height
    pub height: block::Height,

    /// Txs results (might be explicit null)
    pub txs_results: Option<Vec<abci::response::DeliverTx>>,

    /// Events from FinalizeBlock.
    ///
    /// This field is only populated with events since CometBFT version 0.38.
    pub finalize_block_events: Vec<abci::Event>,

    /// Begin block events (might be explicit null)
    ///
    /// This field is not used and set to `None` since CometBFT version 0.38.
    pub begin_block_events: Option<Vec<abci::Event>>,

    /// End block events (might be explicit null)
    ///
    /// This field is not used and set to `None` since CometBFT version 0.38.
    pub end_block_events: Option<Vec<abci::Event>>,

    /// Validator updates (might be explicit null)
    pub validator_updates: Vec<validator::Update>,

    /// New consensus params (might be explicit null)
    pub consensus_param_updates: Option<consensus::Params>,

    /// Merkle hash of the application state
    #[serde(with = "serializers::apphash")]
    pub app_hash: AppHash,
}

pub mod v0_34 {
    use super::Response;
    use crate::dialect::v0_34::Event;
    use crate::prelude::*;
    use crate::{dialect, serializers};
    use serde::{Deserialize, Serialize};
    use tendermint::{block, consensus, validator};

    /// RPC dialect helper for serialization of the response.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct DialectResponse {
        /// Block height
        pub height: block::Height,

        /// Txs results (might be explicit null)
        pub txs_results: Option<Vec<dialect::DeliverTx<Event>>>,

        /// Begin block events (might be explicit null)
        pub begin_block_events: Option<Vec<Event>>,

        /// End block events (might be explicit null)
        pub end_block_events: Option<Vec<Event>>,

        /// Validator updates (might be explicit null)
        #[serde(deserialize_with = "serializers::nullable::deserialize")]
        pub validator_updates: Vec<validator::Update>,

        /// New consensus params (might be explicit null)
        pub consensus_param_updates: Option<consensus::Params>,
    }

    impl crate::Response for DialectResponse {}

    impl From<DialectResponse> for Response {
        fn from(msg: DialectResponse) -> Self {
            Response {
                height: msg.height,
                txs_results: msg
                    .txs_results
                    .map(|v| v.into_iter().map(Into::into).collect()),
                finalize_block_events: vec![],
                begin_block_events: msg
                    .begin_block_events
                    .map(|v| v.into_iter().map(Into::into).collect()),
                end_block_events: msg
                    .end_block_events
                    .map(|v| v.into_iter().map(Into::into).collect()),
                validator_updates: msg.validator_updates,
                consensus_param_updates: msg.consensus_param_updates,
                app_hash: Default::default(),
            }
        }
    }
}

pub mod v0_37 {
    use super::Response;
    use crate::dialect::v0_37::Event;
    use crate::prelude::*;
    use crate::{dialect, serializers};
    use serde::{Deserialize, Serialize};
    use tendermint::{block, consensus, validator, AppHash};

    /// RPC dialect helper for serialization of the response.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct DialectResponse {
        /// Block height
        pub height: block::Height,

        /// Txs results (might be explicit null)
        pub txs_results: Option<Vec<dialect::DeliverTx<Event>>>,

        #[serde(default)]
        pub finalize_block_events: Vec<Event>,

        /// Begin block events (might be explicit null)
        #[serde(default)]
        pub begin_block_events: Option<Vec<Event>>,

        /// End block events (might be explicit null)
        #[serde(default)]
        pub end_block_events: Option<Vec<Event>>,

        /// Validator updates (might be explicit null)
        #[serde(deserialize_with = "serializers::nullable::deserialize")]
        pub validator_updates: Vec<validator::Update>,

        /// New consensus params (might be explicit null)
        pub consensus_param_updates: Option<consensus::Params>,

        #[serde(default)]
        #[serde(with = "serializers::apphash")]
        pub app_hash: AppHash,
    }

    impl crate::Response for DialectResponse {}

    impl From<DialectResponse> for Response {
        fn from(msg: DialectResponse) -> Self {
            Response {
                height: msg.height,
                txs_results: msg
                    .txs_results
                    .map(|v| v.into_iter().map(Into::into).collect()),
                finalize_block_events: msg
                    .finalize_block_events
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                begin_block_events: msg
                    .begin_block_events
                    .map(|v| v.into_iter().map(Into::into).collect()),
                end_block_events: msg
                    .end_block_events
                    .map(|v| v.into_iter().map(Into::into).collect()),
                validator_updates: msg.validator_updates,
                consensus_param_updates: msg.consensus_param_updates,
                app_hash: msg.app_hash,
            }
        }
    }
}
