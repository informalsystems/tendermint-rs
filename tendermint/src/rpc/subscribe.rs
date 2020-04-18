use async_tungstenite::{tokio::connect_async, tokio::TokioAdapter, tungstenite::Message};
use futures::prelude::*;
use tokio::net::TcpStream;

pub struct WebSocketEvents {
    socket: async_tungstenite::WebSocketStream<TokioAdapter<TcpStream>>,
}

pub enum Event {
    GenericEvent { data: serde_json::value::Value },
}

impl WebSocketEvents {
    pub async fn subscribe(url: &str, query: &str) -> Result<Self, Box<dyn std::error::Error>> {
        //TODO support HTTPS
        let (mut ws_stream, _) = connect_async(&format!("ws://{}/subscribe", url)).await?;

        ws_stream.send(Message::text(query)).await?;

        Ok(WebSocketEvents { socket: ws_stream })
    }

    pub async fn next_event(&mut self) -> Result<Event, Box<dyn std::error::Error>> {
        let msg = self
            .socket
            .next()
            .await
            .ok_or_else(|| "web socket closed")??;

        Ok(Event::GenericEvent {
            data: msg.to_string().parse()?,
        })
    }
}
