//! RPC subscription event-related data structures.

use alloc::collections::BTreeMap as HashMap;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tendermint::{abci, Block};

use crate::{dialect, prelude::*, query::EventType, response::Wrapper, serializers, Response};

/// An incoming event produced by a [`Subscription`].
///
/// [`Subscription`]: ../struct.Subscription.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// The query that produced the event.
    pub query: String,
    /// The data associated with the event.
    pub data: EventData,
    /// Event type and attributes map.
    pub events: Option<HashMap<String, Vec<String>>>,
}

// Serialization helper supporting differences in RPC versions.
#[derive(Serialize, Deserialize, Debug)]
pub struct DialectEvent<Ev> {
    /// The query that produced the event.
    pub query: String,
    /// The data associated with the event.
    pub data: DialectEventData<Ev>,
    /// Event type and attributes map.
    pub events: Option<HashMap<String, Vec<String>>>,
}

impl<Ev> Response for DialectEvent<Ev> where Ev: Serialize + DeserializeOwned {}

/// A JSON-RPC-wrapped event.
#[allow(dead_code)]
pub(crate) type WrappedEvent<Ev> = Wrapper<DialectEvent<Ev>>;

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

impl<Ev> From<DialectEvent<Ev>> for Event
where
    Ev: Into<abci::Event>,
{
    fn from(msg: DialectEvent<Ev>) -> Self {
        Event {
            query: msg.query,
            data: msg.data.into(),
            events: msg.events,
        }
    }
}

impl<Ev> From<Event> for DialectEvent<Ev>
where
    abci::Event: Into<Ev>,
{
    fn from(msg: Event) -> Self {
        DialectEvent {
            query: msg.query,
            data: msg.data.into(),
            events: msg.events,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
// To be fixed in 0.24
#[allow(clippy::large_enum_variant)]
pub enum EventData {
    NewBlock {
        block: Option<Block>,
        result_begin_block: Option<abci::response::BeginBlock>,
        result_end_block: Option<abci::response::EndBlock>,
    },
    Tx {
        tx_result: TxInfo,
    },
    GenericJsonEvent(serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "value")]
#[allow(clippy::large_enum_variant)]
pub enum DialectEventData<Ev> {
    #[serde(alias = "tendermint/event/NewBlock")]
    NewBlock {
        block: Option<Block>,
        result_begin_block: Option<dialect::BeginBlock<Ev>>,
        result_end_block: Option<dialect::EndBlock<Ev>>,
    },
    #[serde(alias = "tendermint/event/Tx")]
    Tx {
        #[serde(rename = "TxResult")]
        tx_result: DialectTxInfo<Ev>,
    },
    GenericJsonEvent(serde_json::Value),
}

impl<Ev> From<DialectEventData<Ev>> for EventData
where
    Ev: Into<abci::Event>,
{
    fn from(msg: DialectEventData<Ev>) -> Self {
        match msg {
            DialectEventData::NewBlock {
                block,
                result_begin_block,
                result_end_block,
            } => EventData::NewBlock {
                block,
                result_begin_block: result_begin_block.map(Into::into),
                result_end_block: result_end_block.map(Into::into),
            },
            DialectEventData::Tx { tx_result } => EventData::Tx {
                tx_result: tx_result.into(),
            },
            DialectEventData::GenericJsonEvent(v) => EventData::GenericJsonEvent(v),
        }
    }
}

impl<Ev> From<EventData> for DialectEventData<Ev>
where
    abci::Event: Into<Ev>,
{
    fn from(msg: EventData) -> Self {
        match msg {
            EventData::NewBlock {
                block,
                result_begin_block,
                result_end_block,
            } => DialectEventData::NewBlock {
                block,
                result_begin_block: result_begin_block.map(Into::into),
                result_end_block: result_end_block.map(Into::into),
            },
            EventData::Tx { tx_result } => DialectEventData::Tx {
                tx_result: tx_result.into(),
            },
            EventData::GenericJsonEvent(v) => DialectEventData::GenericJsonEvent(v),
        }
    }
}

/// Transaction result info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxInfo {
    pub height: i64,
    pub index: Option<i64>,
    pub tx: Vec<u8>,
    pub result: TxResult,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DialectTxInfo<Ev> {
    #[serde(with = "serializers::from_str")]
    pub height: i64,
    pub index: Option<i64>,
    #[serde(with = "serializers::bytes::base64string")]
    pub tx: Vec<u8>,
    pub result: DialectTxResult<Ev>,
}

impl<Ev> From<DialectTxInfo<Ev>> for TxInfo
where
    Ev: Into<abci::Event>,
{
    fn from(msg: DialectTxInfo<Ev>) -> Self {
        TxInfo {
            height: msg.height,
            index: msg.index,
            tx: msg.tx,
            result: msg.result.into(),
        }
    }
}

impl<Ev> From<TxInfo> for DialectTxInfo<Ev>
where
    abci::Event: Into<Ev>,
{
    fn from(msg: TxInfo) -> Self {
        DialectTxInfo {
            height: msg.height,
            index: msg.index,
            tx: msg.tx,
            result: msg.result.into(),
        }
    }
}

/// Transaction result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxResult {
    pub log: Option<String>,
    pub gas_wanted: Option<String>,
    pub gas_used: Option<String>,
    pub events: Vec<abci::Event>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DialectTxResult<Ev> {
    pub log: Option<String>,
    pub gas_wanted: Option<String>,
    pub gas_used: Option<String>,
    pub events: Vec<Ev>,
}

impl<Ev> From<DialectTxResult<Ev>> for TxResult
where
    Ev: Into<abci::Event>,
{
    fn from(msg: DialectTxResult<Ev>) -> Self {
        TxResult {
            log: msg.log,
            gas_wanted: msg.gas_wanted,
            gas_used: msg.gas_used,
            events: msg.events.into_iter().map(Into::into).collect(),
        }
    }
}

impl<Ev> From<TxResult> for DialectTxResult<Ev>
where
    abci::Event: Into<Ev>,
{
    fn from(msg: TxResult) -> Self {
        DialectTxResult {
            log: msg.log,
            gas_wanted: msg.gas_wanted,
            gas_used: msg.gas_used,
            events: msg.events.into_iter().map(Into::into).collect(),
        }
    }
}
