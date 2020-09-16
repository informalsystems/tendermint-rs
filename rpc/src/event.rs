//! RPC subscription event-related data structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tendermint::{
    abci::responses::{BeginBlock, EndBlock},
    Block,
};

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum EventData {
    #[serde(alias = "tendermint/event/NewBlock")]
    NewBlock {
        block: Option<Block>,
        result_begin_block: Option<BeginBlock>,
        result_end_block: Option<EndBlock>,
    },
    #[serde(alias = "tendermint/event/Tx")]
    Tx {
        tx_result: TxInfo,
    },
    GenericJSONEvent(serde_json::Value),
}

/// Transaction result info.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TxInfo {
    pub height: String,
    pub index: i64,
    pub tx: String,
    pub result: TxResult,
}

/// Transaction result.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TxResult {
    pub log: String,
    pub gas_wanted: String,
    pub gas_used: String,
    pub events: Vec<TmEvent>,
}

/// Tendermint ABCI Events
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TmEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub attributes: Vec<Attribute>,
}

/// Event Attributes
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}
