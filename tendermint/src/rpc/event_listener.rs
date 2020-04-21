//! Tendermint Websocket event listener client

use crate::{
    net,
    rpc::Request,
    rpc::{endpoint::subscribe, Error as RPCError},
};
use async_tungstenite::{tokio::connect_async, tokio::TokioAdapter, tungstenite::Message};
use futures::prelude::*;
use tokio::net::TcpStream;

/// Event Listener over webocket. https://docs.tendermint.com/master/rpc/#/Websocket/subscribe
pub struct EventListener {
    socket: async_tungstenite::WebSocketStream<TokioAdapter<TcpStream>>,
}

impl EventListener {
    /// Constructor for event listener
    pub async fn connect(
        address: net::Address,
    ) -> Result<EventListener,RPCError> {
        let (host, port) = match &address {
            net::Address::Tcp { host, port, .. } => (host, port),
            other => {
                return Err(
                    RPCError::invalid_params(&format!("invalid RPC address: {:?}", other)).into(),
                );
            }
        };
        //TODO This doesn't have any way to handle a connection over TLS
        let (ws_stream, _unused_tls_stream) =
            connect_async(&format!("ws://{}:{}/websocket", host, port)).await?;
        Ok(EventListener { socket: ws_stream })
    }
    //TODO Have a query type instead of a string
    /// Subscribe to event query stream over the websocket
    pub async fn subscribe(&mut self, query: String) -> Result<(), RPCError> {
        self.socket
            .send(Message::text(
                subscribe::Request::new(query.to_owned()).into_json(),
            ))
            .await?;
        Ok(())
    }

    /// Subscribe to the Events Websocket with a query string for example "tm.event = 'NewBlock'"
    pub async fn get_event(&mut self) -> Result<Event, RPCError> {
        let msg = self
            .socket
            .next()
            .await
            .ok_or_else(|| RPCError::websocket_error("web socket closed"))??;

        match msg.to_string().parse::<serde_json::Value>() {
            Ok(data) => Ok(Event::GenericJSONEvent { data }),
            Err(_) => Ok(Event::GenericStringEvent {
                data: msg.to_string(),
            }),
        }
    }
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
