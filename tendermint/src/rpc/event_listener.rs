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

use crate::rpc::error::Code;
use tokio::net::TcpStream;

/// There are only two valid queries to the websocket. A query that subscribes to all transactions
/// and a query that susbscribes to all blocks.
#[derive(Serialize, Deserialize, Debug, Clone)]
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
        // TODO(ismail): this works if subscriptions are fired sequentially and no event or
        // ping message gets in the way:
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
    pub async fn get_event(&mut self) -> Result<Option<ResultEvent>, RPCError> {
        let msg = self
            .socket
            .next()
            .await
            .ok_or_else(|| RPCError::websocket_error("web socket closed"))??;

        if let Ok(result_event) = serde_json::from_str::<WrappedResultEvent>(&msg.to_string()) {
            // if we get an rpc error here, we will bubble it up:
            return Ok(Some(result_event.into_result()?));
        }
        dbg!("We did not receive a valid JSONRPC wrapped ResultEvent!");
        if serde_json::from_str::<String>(&msg.to_string()).is_ok() {
            // FIXME(ismail): Until this is a proper websocket client
            // (or the endpoint moved to grpc in tendermint), we accept whatever was read here
            // dbg! it out and return None below.
            dbg!("Instead of JSONRPC wrapped ResultEvent, we got:");
            dbg!(&msg.to_string());
            return Ok(None);
        }
        dbg!("received neither event nor generic string message:");
        dbg!(&msg.to_string());
        Err(RPCError::new(
            Code::Other(-1),
            Some("received neither event nor generic string message".to_string()),
        ))
    }
}

// TODO(ismail): this should live somewhere else; these events are also
// published by the event bus independent from RPC.
// We leave it here for now because unsupported types are still
// decodeable via fallthrough variants (GenericJSONEvent).
/// The Event enum is typed events emitted by the Websockets.
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

    /// Generic event containing json data
    GenericJSONEvent(
        /// generic event json data
        serde_json::Value,
    ),
}

/// Event data from a subscription
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultEvent {
    /// Query for this result
    pub query: String,
    /// Tendermint EventData
    pub data: TMEventData,
    /// Event type and event attributes map
    pub events: Option<HashMap<String, Vec<String>>>,
}
impl response::Response for ResultEvent {}

/// JSONRPC wrapped ResultEvent
pub type WrappedResultEvent = Wrapper<ResultEvent>;

/// TX value
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventDataTx {
    /// The actual TxResult
    #[serde(rename = "TxResult")]
    pub tx_result: TxResult,
}

/// Tx Result
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxResult {
    pub height: String,
    pub index: i64,
    pub tx: String,
    pub result: TxResultResult,
}

/// TX Results Results
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxResultResult {
    pub log: String,
    pub gas_wanted: String,
    pub gas_used: String,
    pub events: Vec<TmEvent>,
}
impl response::Response for TxResultResult {}

/// Tendermint ABCI Events
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TmEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub attributes: Vec<Attribute>,
}
/// Event Attributes
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}

///Block Value
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventDataNewBlock {
    pub block: Option<Block>,

    // TODO(ismail): these should be the same as abci::responses::BeginBlock
    // and abci::responses::EndBlock
    pub result_begin_block: Option<ResultBeginBlock>,
    pub result_end_block: Option<ResultEndBlock>,
}

/// Begin Block Events
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultBeginBlock {
    pub events: Option<Vec<TmEvent>>,
}
///End Block Events
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultEndBlock {
    pub validator_updates: Option<Vec<Option<serde_json::Value>>>,
}
