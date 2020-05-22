//! Tendermint Websocket event listener client

use crate::{
    net,
    rpc::Request,
    rpc::{endpoint::subscribe, Error as RPCError},
};
use async_tungstenite::{tokio::connect_async, tokio::TokioAdapter, tungstenite::Message};
use futures::prelude::*;
use serde::{Deserialize, Serialize};

use tokio::net::TcpStream;

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
    pub async fn subscribe(&mut self, query: &str) -> Result<(), RPCError> {
        self.socket
            .send(Message::text(
                subscribe::Request::new(query.to_owned()).into_json(),
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

        match serde_json::from_str::<JSONRPC>(&msg.to_string()) {
            Ok(data) => Ok(Event::JsonRPCTransactionResult {
                data: Box::new(data),
            }),

            Err(_) => match msg.to_string().parse::<serde_json::Value>() {
                Ok(data) => Ok(Event::GenericJSONEvent { data }),
                Err(_) => Ok(Event::GenericStringEvent {
                    data: msg.to_string(),
                }),
            },
        }
    }
}

//TODO more event types
/// The Event enum is typed events emitted by the Websockets
#[derive(Debug, Clone)]
pub enum Event {
    /// The result of the ABCI app processing a transaction, serialized as JSON RPC response
    JsonRPCTransactionResult {
        /// the tx result data
        data: Box<JSONRPC>,
    },

    ///Generic event containing json data
    GenericJSONEvent {
        /// generic event json data
        data: serde_json::Value,
    },
    ///Generic String Event
    GenericStringEvent {
        /// generic string data
        data: String,
    },
}

/// Standard JSON RPC Wrapper
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JSONRPC {
    jsonrpc: String,
    id: String,
    result: RPCResult,
}
/// JSON RPC Result Type
#[derive(Serialize, Deserialize, Debug, Clone)]
struct RPCResult {
    query: String,
    data: Data,
    events: std::collections::HashMap<String, Vec<String>>,
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
    events: Vec<TxEvent>,
}

/// Tx Events
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TxEvent {
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

impl JSONRPC {
    /// Extract events from TXEvent if event matches are type query
    pub fn extract_events(
        &self,
        action_query: &str,
    ) -> Result<std::collections::HashMap<String, Vec<String>>, &'static str> {
        let events = &self.result.events;
        if let Some(message_action) = events.get("message.action") {
            if message_action.contains(&action_query.to_owned()) {
                return Ok(events.clone());
            }
        }

        Err("Incorrect Event Type")
    }
}
