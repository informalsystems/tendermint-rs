//! RPC subscription event-related data structures.

use alloc::collections::BTreeMap as HashMap;
use serde::{Deserialize, Serialize};
use tendermint::abci::responses::{BeginBlock, EndBlock};
use tendermint::Block;

use crate::prelude::*;
use crate::query::EventType;
use crate::{response::Wrapper, Response};

/// An incoming event produced by a [`Subscription`].
///
/// [`Subscription`]: ../struct.Subscription.html
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Event {
    /// The query that produced the event.
    pub query: String,
    /// The data associated with the event.
    pub data: EventData,
    /// Event type and attributes map.
    pub events: Option<HashMap<String, Vec<String>>>,
}
impl Response for Event {}

/// A JSON-RPC-wrapped event.
pub type WrappedEvent = Wrapper<Event>;

impl Event {
    /// Returns the type associated with this event, if we recognize it.
    ///
    /// Returns `None` if we don't yet support this event type.
    pub fn event_type(&self) -> Option<EventType> {
        match self.data {
            EventData::NewBlock { .. } => Some(EventType::NewBlock),
            EventData::Tx { .. } => Some(EventType::Tx),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", content = "value")]
// To be fixed in 0.24
#[allow(clippy::large_enum_variant)]
pub enum EventData {
    #[serde(alias = "tendermint/event/NewBlock")]
    NewBlock {
        block: Option<Block>,
        result_begin_block: Option<BeginBlock>,
        result_end_block: Option<EndBlock>,
    },
    #[serde(alias = "tendermint/event/Tx")]
    Tx {
        #[serde(rename = "TxResult")]
        tx_result: TxInfo,
    },
    GenericJsonEvent(serde_json::Value),
}

/// Transaction result info.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TxInfo {
    #[serde(with = "tendermint_proto::serializers::from_str")]
    pub height: i64,
    pub index: Option<i64>,
    #[serde(with = "tendermint_proto::serializers::bytes::base64string")]
    pub tx: Vec<u8>,
    pub result: TxResult,
}

/// Transaction result.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TxResult {
    pub log: Option<String>,
    pub gas_wanted: Option<String>,
    pub gas_used: Option<String>,
    pub events: Vec<tendermint::abci::Event>,
}
