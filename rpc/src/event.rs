//! RPC subscription event-related data structures.

use alloc::collections::BTreeMap as HashMap;

use tendermint::{abci, block, Block};

use crate::{prelude::*, query::EventType};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventData {
    /// Data of the newly committed block.
    ///
    /// Used since CometBFT 0.38.
    NewBlock {
        block: Option<Box<Block>>,
        block_id: block::Id,
        result_finalize_block: Option<abci::response::FinalizeBlock>,
    },
    /// Data of the newly committed block.
    ///
    /// Used in CometBFT versions before 0.38.
    LegacyNewBlock {
        block: Option<Box<Block>>,
        result_begin_block: Option<abci::response::BeginBlock>,
        result_end_block: Option<abci::response::EndBlock>,
    },
    Tx {
        tx_result: TxInfo,
    },
    GenericJsonEvent(serde_json::Value),
}

/// Transaction result info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxInfo {
    pub height: i64,
    pub index: Option<i64>,
    pub tx: Vec<u8>,
    pub result: TxResult,
}

/// Transaction result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxResult {
    pub log: Option<String>,
    pub gas_wanted: Option<String>,
    pub gas_used: Option<String>,
    pub events: Vec<abci::Event>,
}

/// Serialization helpers for CometBFT 0.34 RPC
pub mod v0_34 {
    use super::{Event, EventData, TxInfo, TxResult};
    use crate::dialect::v0_34::Event as RpcEvent;
    use crate::prelude::*;
    use crate::{dialect, serializers, Response};
    use alloc::collections::BTreeMap as HashMap;
    use serde::{Deserialize, Serialize};
    use tendermint::Block;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DialectEvent {
        /// The query that produced the event.
        pub query: String,
        /// The data associated with the event.
        pub data: DialectEventData,
        /// Event type and attributes map.
        pub events: Option<HashMap<String, Vec<String>>>,
    }

    impl Response for DialectEvent {}

    impl From<DialectEvent> for Event {
        fn from(msg: DialectEvent) -> Self {
            Event {
                query: msg.query,
                data: msg.data.into(),
                events: msg.events,
            }
        }
    }

    impl From<Event> for DialectEvent {
        fn from(msg: Event) -> Self {
            DialectEvent {
                query: msg.query,
                data: msg.data.into(),
                events: msg.events,
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(tag = "type", content = "value")]
    #[allow(clippy::large_enum_variant)]
    pub enum DialectEventData {
        #[serde(alias = "tendermint/event/NewBlock")]
        NewBlock {
            block: Option<Box<Block>>,
            result_begin_block: Option<dialect::BeginBlock<RpcEvent>>,
            result_end_block: Option<dialect::EndBlock<RpcEvent>>,
        },
        #[serde(alias = "tendermint/event/Tx")]
        Tx {
            #[serde(rename = "TxResult")]
            tx_result: DialectTxInfo,
        },
        GenericJsonEvent(serde_json::Value),
    }

    impl From<DialectEventData> for EventData {
        fn from(msg: DialectEventData) -> Self {
            match msg {
                DialectEventData::NewBlock {
                    block,
                    result_begin_block,
                    result_end_block,
                } => EventData::LegacyNewBlock {
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

    impl From<EventData> for DialectEventData {
        fn from(msg: EventData) -> Self {
            match msg {
                EventData::LegacyNewBlock {
                    block,
                    result_begin_block,
                    result_end_block,
                } => DialectEventData::NewBlock {
                    block,
                    result_begin_block: result_begin_block.map(Into::into),
                    result_end_block: result_end_block.map(Into::into),
                },
                // This variant should not be used with 0.34, but we're using
                // this impl only for the mock server.
                EventData::NewBlock {
                    block,
                    block_id: _,
                    result_finalize_block: _,
                } => DialectEventData::NewBlock {
                    block,
                    result_begin_block: None,
                    result_end_block: None,
                },
                EventData::Tx { tx_result } => DialectEventData::Tx {
                    tx_result: tx_result.into(),
                },
                EventData::GenericJsonEvent(v) => DialectEventData::GenericJsonEvent(v),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DialectTxInfo {
        #[serde(with = "serializers::from_str")]
        pub height: i64,
        pub index: Option<i64>,
        #[serde(with = "serializers::bytes::base64string")]
        pub tx: Vec<u8>,
        pub result: DialectTxResult,
    }

    impl From<DialectTxInfo> for TxInfo {
        fn from(msg: DialectTxInfo) -> Self {
            TxInfo {
                height: msg.height,
                index: msg.index,
                tx: msg.tx,
                result: msg.result.into(),
            }
        }
    }

    impl From<TxInfo> for DialectTxInfo {
        fn from(msg: TxInfo) -> Self {
            DialectTxInfo {
                height: msg.height,
                index: msg.index,
                tx: msg.tx,
                result: msg.result.into(),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DialectTxResult {
        pub log: Option<String>,
        pub gas_wanted: Option<String>,
        pub gas_used: Option<String>,
        pub events: Vec<RpcEvent>,
    }

    impl From<DialectTxResult> for TxResult {
        fn from(msg: DialectTxResult) -> Self {
            TxResult {
                log: msg.log,
                gas_wanted: msg.gas_wanted,
                gas_used: msg.gas_used,
                events: msg.events.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl From<TxResult> for DialectTxResult {
        fn from(msg: TxResult) -> Self {
            DialectTxResult {
                log: msg.log,
                gas_wanted: msg.gas_wanted,
                gas_used: msg.gas_used,
                events: msg.events.into_iter().map(Into::into).collect(),
            }
        }
    }
}

/// Serialization helpers for the RPC protocol used since CometBFT 0.37
pub mod latest {
    use super::{Event, EventData, TxInfo, TxResult};
    use crate::prelude::*;
    use crate::{serializers, Response};
    use alloc::collections::BTreeMap as HashMap;
    use serde::{Deserialize, Serialize};
    use tendermint::abci::Event as RpcEvent;
    use tendermint::{abci, block, Block};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DialectEvent {
        /// The query that produced the event.
        pub query: String,
        /// The data associated with the event.
        pub data: DialectEventData,
        /// Event type and attributes map.
        pub events: Option<HashMap<String, Vec<String>>>,
    }

    impl Response for DialectEvent {}

    impl From<DialectEvent> for Event {
        fn from(msg: DialectEvent) -> Self {
            Event {
                query: msg.query,
                data: msg.data.into(),
                events: msg.events,
            }
        }
    }

    impl From<Event> for DialectEvent {
        fn from(msg: Event) -> Self {
            DialectEvent {
                query: msg.query,
                data: msg.data.into(),
                events: msg.events,
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(tag = "type", content = "value")]
    #[allow(clippy::large_enum_variant)]
    pub enum DialectEventData {
        #[serde(alias = "tendermint/event/NewBlock")]
        NewBlock {
            block: Option<Box<Block>>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            result_begin_block: Option<abci::response::BeginBlock>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            result_end_block: Option<abci::response::EndBlock>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            block_id: Option<block::Id>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            result_finalize_block: Option<abci::response::FinalizeBlock>,
        },
        #[serde(alias = "tendermint/event/Tx")]
        Tx {
            #[serde(rename = "TxResult")]
            tx_result: DialectTxInfo,
        },
        GenericJsonEvent(serde_json::Value),
    }

    impl From<DialectEventData> for EventData {
        fn from(msg: DialectEventData) -> Self {
            match msg {
                DialectEventData::NewBlock {
                    block,
                    block_id: Some(block_id),
                    result_finalize_block,
                    result_begin_block: _,
                    result_end_block: _,
                } => EventData::NewBlock {
                    block,
                    block_id,
                    result_finalize_block,
                },
                DialectEventData::NewBlock {
                    block,
                    result_begin_block,
                    result_end_block,
                    block_id: None,
                    result_finalize_block: _,
                } => EventData::LegacyNewBlock {
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

    impl From<EventData> for DialectEventData {
        fn from(msg: EventData) -> Self {
            match msg {
                EventData::NewBlock {
                    block,
                    block_id,
                    result_finalize_block,
                } => DialectEventData::NewBlock {
                    block,
                    block_id: Some(block_id),
                    result_finalize_block,
                    result_begin_block: None,
                    result_end_block: None,
                },
                // This variant should not be used since 0.38, but we're using
                // this impl only for the mock server.
                EventData::LegacyNewBlock {
                    block,
                    result_begin_block,
                    result_end_block,
                } => DialectEventData::NewBlock {
                    block,
                    block_id: None,
                    result_finalize_block: None,
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

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DialectTxInfo {
        #[serde(with = "serializers::from_str")]
        pub height: i64,
        pub index: Option<i64>,
        #[serde(with = "serializers::bytes::base64string")]
        pub tx: Vec<u8>,
        pub result: DialectTxResult,
    }

    impl From<DialectTxInfo> for TxInfo {
        fn from(msg: DialectTxInfo) -> Self {
            TxInfo {
                height: msg.height,
                index: msg.index,
                tx: msg.tx,
                result: msg.result.into(),
            }
        }
    }

    impl From<TxInfo> for DialectTxInfo {
        fn from(msg: TxInfo) -> Self {
            DialectTxInfo {
                height: msg.height,
                index: msg.index,
                tx: msg.tx,
                result: msg.result.into(),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DialectTxResult {
        pub log: Option<String>,
        pub gas_wanted: Option<String>,
        pub gas_used: Option<String>,
        pub events: Vec<RpcEvent>,
    }

    impl From<DialectTxResult> for TxResult {
        fn from(msg: DialectTxResult) -> Self {
            TxResult {
                log: msg.log,
                gas_wanted: msg.gas_wanted,
                gas_used: msg.gas_used,
                events: msg.events.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl From<TxResult> for DialectTxResult {
        fn from(msg: TxResult) -> Self {
            DialectTxResult {
                log: msg.log,
                gas_wanted: msg.gas_wanted,
                gas_used: msg.gas_used,
                events: msg.events.into_iter().map(Into::into).collect(),
            }
        }
    }
}

pub mod v0_37 {
    pub use super::latest::*;
}

pub mod v0_38 {
    pub use super::latest::*;
}
