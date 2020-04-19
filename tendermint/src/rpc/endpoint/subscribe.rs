//! `/subscribe` endpoint JSONRPC wrappera

use crate::rpc;
use crate::rpc::Request as RQ;
use async_tungstenite::{tokio::connect_async, tokio::TokioAdapter, tungstenite::Message};
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::Read;
use tokio::net::TcpStream;

/// Subscribe request for events on websocket
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    query: String,
}

impl Request {
    /// List validators for a specific block
    pub fn new(query: String) -> Self {
        Self { query }
    }
}

impl rpc::Request for Request {
    type Response = Response;

    fn method(&self) -> rpc::Method {
        rpc::Method::Subscribe
    }
}

/// Status responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {}

impl rpc::Response for Response {
    /// We throw away response data JSON string so swallow errors and return the empty Response
    fn from_string(_response: impl AsRef<[u8]>) -> Result<Self, rpc::Error> {
        Ok(Response {})
    }

    /// We throw away responses in `subscribe` so swallow errors from the `io::Reader` and provide
    /// the Response
    fn from_reader(_reader: impl Read) -> Result<Self, rpc::Error> {
        Ok(Response {})
    }
}

/// WebsocketEvents are pollable struct getting events from the Tendermint websocket
pub struct WebSocketEvents {
    socket: async_tungstenite::WebSocketStream<TokioAdapter<TcpStream>>,
}

//TODO more event types
/// The Event enum is typed events emmitted by the Websockets
#[derive(Debug)]
pub enum Event {
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

impl WebSocketEvents {
    /// Connect to the Tendermint websocket
    pub async fn websocket(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        //TODO support HTTPS
        let (ws_stream, _) = connect_async(&format!("{}/websocket", url)).await?;
        Ok(WebSocketEvents { socket: ws_stream })
    }

    /// Send JSON RPC with query subscription over WebSocket
    pub async fn subscribe(&mut self, query: &str) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self
            .socket
            .send(Message::text(Request::new(query.to_owned()).into_json()))
            .await?;
        Ok(())
    }

    /// Poll next event to get events from the websocket
    pub async fn next_event(&mut self) -> Result<Event, Box<dyn std::error::Error>> {
        let msg = self
            .socket
            .next()
            .await
            .ok_or_else(|| "web socket closed")??;

        match msg.to_string().parse::<serde_json::Value>() {
            Ok(data) => Ok(Event::GenericJSONEvent { data }),
            Err(_) => Ok(Event::GenericStringEvent {
                data: msg.to_string(),
            }),
        }
    }
}
