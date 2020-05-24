//! Tendermint Websocket event listener client

use crate::{
    block::Block,
    net,
    rpc::response,
    rpc::response::Wrapper,
    rpc::Request,
    rpc::{endpoint::subscribe, Error as RPCError},
};
use async_tungstenite::{tokio::connect_async, tokio::TokioAdapter, tungstenite::Message};
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error as stdError;

use tokio::net::TcpStream;
/// There are only two valid queries to the websocket. A query that subscribes to all transactions
/// and a query that susbscribes to all blocks.
pub enum EventSubscription {
    /// Subscribe to all transactions
    TransactionSubscription,
    ///Subscribe to all blocks
    BlockSubscription,
}

impl EventSubscription {
    ///Convert the query enum to a string
    pub fn as_str(&self) -> &str {
        match self {
            EventSubscription::TransactionSubscription => "tm.event='Tx'",
            EventSubscription::BlockSubscription => "tm.event='NewBlock'",
        }
    }
}

/// Event Listener over websocket.
/// See: <https://docs.tendermint.com/master/rpc/#/Websocket/subscribe>
pub struct EventListener {
    socket: async_tungstenite::WebSocketStream<TokioAdapter<TcpStream>>,
}

impl EventListener {
    /// Constructor for event listener
    pub async fn connect(address: net::Address) -> Result<EventListener, RPCError> {
        let (host, port) = match address {
            net::Address::Tcp { host, port, .. } => (host, port),
            other => {
                return Err(RPCError::invalid_params(&format!(
                    "invalid RPC address: {:?}",
                    other
                )));
            }
        };
        //TODO This doesn't have any way to handle a connection over TLS
        let (ws_stream, _unused_tls_stream) =
            connect_async(&format!("ws://{}:{}/websocket", host, port)).await?;
        Ok(EventListener { socket: ws_stream })
    }

    /// Subscribe to event query stream over the websocket
    pub async fn subscribe(&mut self, query: EventSubscription) -> Result<(), Box<dyn stdError>> {
        self.socket
            .send(Message::text(
                subscribe::Request::new(query.as_str().to_owned()).into_json(),
            ))
            .await?;
        // Wait for an empty response on subscribe
        let msg = self
            .socket
            .next()
            .await
            .ok_or_else(|| RPCError::websocket_error("web socket closed"))??;
        serde_json::from_str::<Wrapper<subscribe::Response>>(&msg.to_string())?.into_result()?;

        Ok(())
    }

    /// Get the next event from the websocket
    pub async fn get_event(&mut self) -> Result<Option<TMEventData>, Box<dyn stdError>> {
        let msg = self
            .socket
            .next()
            .await
            .ok_or_else(|| RPCError::websocket_error("web socket closed"))??;
        dbg!("get_event msg:");
        dbg!(&msg.to_string());
        let result_event =
            serde_json::from_str::<WrappedResultEvent>(&msg.to_string())?.into_result()?;

        Ok(result_event.data)
    }
}

// TODO: (later) this should live somewhere else; these events are also
// published byte the event bus independent from RPC.
// We leave it here for now because unsupported types are still
// decodeable via fallthrough variants (GenericJSONEvent).
/// The Event enum is typed events emitted by the Websockets
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "value")]
#[allow(clippy::large_enum_variant)]
pub enum TMEventData {
    /// EventDataNewBlock is returned upon subscribing to "tm.event='NewBlock'"
    #[serde(alias = "tendermint/event/NewBlock")]
    EventDataNewBlock(EventDataNewBlock),

    /// EventDataTx is returned upon subscribing to "tm.event='Tx'"
    #[serde(alias = "tendermint/event/Tx")]
    EventDataTx(EventDataTx),

    ///Generic event containing json data
    GenericJSONEvent(
        /// generic event json data
        serde_json::Value,
    ),
}

/// Event data from a subscription
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultEvent {
    query: Option<String>,
    data: Option<TMEventData>,
    events: Option<HashMap<String, Vec<String>>>,
}
impl response::Response for ResultEvent {}

/// JSONRPC wrapped ResultEvent
pub type WrappedResultEvent = Wrapper<ResultEvent>;

/// TX value
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventDataTx {
    #[serde(rename = "TxResult")]
    tx_result: TxResult,
}

/// Tx Result
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxResult {
    height: String,
    index: i64,
    tx: String,
    result: TxResultResult,
}

/// TX Results Results
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxResultResult {
    log: String,
    gas_wanted: String,
    gas_used: String,
    events: Vec<TmEvent>,
}
impl response::Response for TxResultResult {}

/// Tendermint ABCI Events
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TmEvent {
    #[serde(rename = "type")]
    event_type: String,
    attributes: Vec<Attribute>,
}
/// Event Attributes
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attribute {
    key: String,
    value: String,
}

///Block Value
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventDataNewBlock {
    block: Option<Block>,

    // TODO these should be the same as abci::responses::BeginBlock
    // and abci::responses::EndBlock
    result_begin_block: Option<ResultBeginBlock>,
    result_end_block: Option<ResultEndBlock>,
}

/// Begin Block Events
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultBeginBlock {
    events: Option<Vec<TmEvent>>,
}
///End Block Events
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultEndBlock {
    validator_updates: Option<Vec<Option<serde_json::Value>>>,
}
