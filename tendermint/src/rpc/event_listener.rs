//! Tendermint Websocket event listener client

use crate::{
    block::Block,
    net,
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
/// See: https://docs.tendermint.com/master/rpc/#/Websocket/subscribe
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
    //TODO Have a query type instead of a string
    /// Subscribe to event query stream over the websocket
    pub async fn subscribe(&mut self, query: EventSubscription) -> Result<(), RPCError> {
        self.socket
            .send(Message::text(
                subscribe::Request::new(query.as_str().to_owned()).into_json(),
            ))
            .await?;
        Ok(())
    }

    /// Get the next event from the websocket
    pub async fn get_event(&mut self) -> Result<Event, RPCError> {
        let msg = self
            .socket
            .next()
            .await
            .ok_or_else(|| RPCError::websocket_error("web socket closed"))??;
        match serde_json::from_str::<JsonRPCBlockResult>(&msg.to_string()) {
            Ok(data) => {
                if let Some(data) = data.0.result {
                    Ok(Event::JsonRPCBlockResult(data))
                } else {
                    // The Websocket should never send an empty block
                    panic!("Websocket sent us an empty block")
                }
            }
            Err(_) => match serde_json::from_str::<JsonRPCTransactionResult>(&msg.to_string()) {
                Ok(data) => {
                    if let Some(data) = data.0.result {
                        Ok(Event::JsonRPCTransactionResult(data))
                    } else {
                        // The Websocket should never send an empty transaction
                        panic!("Websocket sent us an empty transaction")
                    }
                }
                Err(_) => match serde_json::from_str::<serde_json::Value>(&msg.to_string()) {
                    Ok(data) => Ok(Event::GenericJSONEvent(data)),
                    Err(_) => Ok(Event::GenericStringEvent(msg.to_string())),
                },
            },
        }
    }
}
/// The Event enum is typed events emmitted by the Websockets
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Event {
    /// The result of the ABCI app processing a transaction, serialized as JSON RPC response
    JsonRPCBlockResult(
        /// The Block Result
        RPCBlockResult,
    ),

    /// The result of the ABCI app processing a transaction, serialized as JSON RPC response
    JsonRPCTransactionResult(
        /// the tx result data
        RPCTxResult,
    ),

    ///Generic event containing json data
    GenericJSONEvent(
        /// generic event json data
        serde_json::Value,
    ),
    ///Generic String Event
    GenericStringEvent(
        /// generic string data
        String,
    ),
}

/// Websocket result for Processed Transactions
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRPCTransactionResult(Wrapper<RPCTxResult>);
/// Websocket result for Processed Block
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRPCBlockResult(Wrapper<RPCBlockResult>);

/// JSON RPC Result Type
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RPCTxResult {
    query: String,
    data: Data,
    events: HashMap<String, Vec<String>>,
}

/// TX data
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Data {
    #[serde(rename = "type")]
    data_type: String,
    value: TxValue,
}
/// TX value
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TxValue {
    #[serde(rename = "TxResult")]
    tx_result: TxResult,
}
/// Tx Result
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TxResult {
    height: String,
    index: i64,
    tx: String,
    result: TxResultResult,
}
/// TX Results Results
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TxResultResult {
    log: String,
    gas_wanted: String,
    gas_used: String,
    events: Vec<TmEvent>,
}

/// Tendermint ABCI Events
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TmEvent {
    #[serde(rename = "type")]
    event_type: String,
    attributes: Vec<Attribute>,
}
/// Event Attributes
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Attribute {
    key: String,
    value: String,
}

/// Block Results
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RPCBlockResult {
    query: String,
    data: BlockResultData,
    events: HashMap<String, Vec<String>>,
}
/// Block Results data
#[derive(Serialize, Deserialize, Debug, Clone)]
struct BlockResultData {
    #[serde(rename = "type")]
    data_type: String,
    value: BlockValue,
}
///Block Value
#[derive(Serialize, Deserialize, Debug, Clone)]
struct BlockValue {
    block: Block,
    result_begin_block: ResultBeginBlock,
    result_end_block: ResultEndBlock,
}

/// Begin Block Events
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultBeginBlock {
    events: Vec<TmEvent>,
}
///End Block Events
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultEndBlock {
    validator_updates: Vec<Option<serde_json::Value>>,
}

impl JsonRPCTransactionResult {
    /// Extract events from TXEvent if event matches are type query
    pub fn extract_events(
        &self,
        action_query: &str,
    ) -> Result<HashMap<String, Vec<String>>, Box<dyn stdError>> {
        match &self.0.result {
            Some(ref result) => {
                if let Some(message_action) = result.events.get("message.action") {
                    if message_action.contains(&action_query.to_owned()) {
                        return Ok(result.events.clone());
                    }
                }
                Err("Incorrect Event Type".into())
            }

            None => Err("No result data found".into()),
        }
    }
}
