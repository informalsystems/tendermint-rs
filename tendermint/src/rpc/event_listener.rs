//! Tendermint Websocket event listener client

use crate::{
    net,
    rpc::Request,
    rpc::{endpoint::subscribe, Error as RPCError},
};
use async_tungstenite::{tokio::connect_async, tokio::TokioAdapter, tungstenite::Message};
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Event Listener over webocket. https://docs.tendermint.com/master/rpc/#/Websocket/subscribe
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
        match serde_json::from_str::<JSONRpcBlockResult>(&msg.to_string()) {
            Ok(data) => Ok(Event::JsonRPCBlockResult(data)),
            Err(_) => match serde_json::from_str::<JSONRpcTxResult>(&msg.to_string()) {
                Ok(data) => Ok(Event::JsonRPCTransactionResult(data)),
                Err(_) => match serde_json::from_str::<serde_json::Value>(&msg.to_string()) {
                    Ok(data) => Ok(Event::GenericJSONEvent(data)),
                    Err(_) => Ok(Event::GenericStringEvent(msg.to_string())),
                },
            },
        }
    }
}
//TODO more event types
/// The Event enum is typed events emmitted by the Websockets
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Event {
    /// The result of the ABCI app processing a transaction, serialized as JSON RPC response
    JsonRPCBlockResult(JSONRpcBlockResult),

    /// The result of the ABCI app processing a transaction, serialized as JSON RPC response
    JsonRPCTransactionResult(
        /// the tx result data
        JSONRpcTxResult,
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

/// Standard JSON RPC Wrapper
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JSONRpcTxResult {
    jsonrpc: String,
    id: String,
    result: RPCResult,
}
/// JSON RPC Result Type
#[derive(Serialize, Deserialize, Debug, Clone)]
struct RPCResult {
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

///Block Results JSON PRC response
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JSONRpcBlockResult {
    jsonrpc: String,
    id: String,
    result: BlockResult,
}
/// Block Results
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockResult {
    query: String,
    data: BlockResultData,
    events: HashMap<String, Vec<String>>,
}
/// Block Results data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockResultData {
    #[serde(rename = "type")]
    data_type: String,
    value: BlockValue,
}
///Block Value
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockValue {
    block: Block,
    result_begin_block: ResultBeginBlock,
    result_end_block: ResultEndBlock,
}
/// Block
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    header: Header,
    data: BlockData,
    evidence: Evidence,
    last_commit: LastCommit,
}
///Block Txs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockData {
    txs: Option<serde_json::Value>,
}
///Tendermint evidence
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Evidence {
    evidence: Option<serde_json::Value>,
}
/// Header
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    version: Version,
    chain_id: String,
    height: String,
    time: String,
    last_block_id: BlockId,
    last_commit_hash: String,
    data_hash: String,
    validators_hash: String,
    next_validators_hash: String,
    consensus_hash: String,
    app_hash: String,
    last_results_hash: String,
    evidence_hash: String,
    proposer_address: String,
}
///Block ID
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockId {
    hash: String,
    parts: Parts,
}
/// Block Parts
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parts {
    total: String,
    hash: String,
}
///Block version
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Version {
    block: String,
    app: String,
}
///Perevious Commit
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LastCommit {
    height: String,
    round: String,
    block_id: BlockId,
    signatures: Vec<Signature>,
}
///Signatures
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Signature {
    block_id_flag: i64,
    validator_address: String,
    timestamp: String,
    signature: String,
}
/// Begin Blocke Envts
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultBeginBlock {
    events: Vec<TmEvent>,
}
///End Block Events
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultEndBlock {
    validator_updates: Vec<Option<serde_json::Value>>,
}

impl JSONRpcTxResult {
    /// Extract events from TXEvent if event matches are type query
    pub fn extract_events(
        &self,
        action_query: &str,
    ) -> Result<HashMap<String, Vec<String>>, &'static str> {
        let events = &self.result.events;
        if let Some(message_action) = events.get("message.action") {
            if message_action.contains(&action_query.to_owned()) {
                return Ok(events.clone());
            }
        }

        Err("Incorrect Event Type")
    }
}
