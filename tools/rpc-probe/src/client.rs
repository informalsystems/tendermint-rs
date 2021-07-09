//! WebSocket client for interacting with a remote Tendermint node.
//!
//! We purposefully do not use the WebSocket client provided by the
//! `tendermint-rpc` crate because we need to record the raw JSON-RPC responses
//! from the remote endpoint. The `tendermint-rpc` client does not expose these
//! raw responses.

use crate::error::{Error, Result};
use crate::subscription::{SubscriptionRx, SubscriptionTx};
use crate::utils::uuid_v4;
use async_tungstenite::tokio::{connect_async, ConnectStream};
use async_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use async_tungstenite::tungstenite::protocol::CloseFrame;
use async_tungstenite::tungstenite::Message;
use async_tungstenite::WebSocketStream;
use futures::{SinkExt, StreamExt};
use log::{debug, error, warn};
use serde_json::json;
use std::borrow::Cow;
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Client {
    cmd_tx: DriverCommandTx,
}

impl Client {
    pub async fn new(url: &str) -> Result<(Self, ClientDriver)> {
        let (stream, _response) = connect_async(url).await?;
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        Ok((Client { cmd_tx }, ClientDriver::new(stream, cmd_rx)))
    }

    pub async fn request(&mut self, wrapper: &serde_json::Value) -> Result<serde_json::Value> {
        let (response_tx, mut response_rx) = mpsc::unbounded_channel();
        self.send_cmd(DriverCommand::Request {
            wrapper: wrapper.clone(),
            response_tx,
        })
        .await?;
        response_rx.recv().await.ok_or_else(|| {
            Error::InternalError("internal channel communication problem".to_string())
        })?
    }

    pub async fn subscribe(
        &mut self,
        id: &str,
        query: &str,
    ) -> Result<(SubscriptionRx, serde_json::Value)> {
        let (subscription_tx, subscription_rx) = mpsc::unbounded_channel();
        let (response_tx, mut response_rx) = mpsc::unbounded_channel();
        self.send_cmd(DriverCommand::Subscribe {
            id: id.to_owned(),
            query: query.to_owned(),
            subscription_tx,
            response_tx,
        })
        .await?;
        let response = response_rx.recv().await.ok_or_else(|| {
            Error::InternalError("internal channel communication problem".to_string())
        })??;
        Ok((subscription_rx, response))
    }

    /// Blocks the current async task until the blockchain reaches *at least*
    /// the given target height.
    pub async fn wait_for_height(&mut self, h: u64) -> Result<()> {
        let (mut subs, _) = self.subscribe(&uuid_v4(), "tm.event = 'NewBlock'").await?;
        while let Some(result) = subs.recv().await {
            let resp = result?;
            // TODO(thane): Find a more readable way of getting this value.
            let height = resp
                .get("result")
                .unwrap()
                .get("data")
                .unwrap()
                .get("value")
                .unwrap()
                .get("block")
                .unwrap()
                .get("header")
                .unwrap()
                .get("height")
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
                .parse::<u64>()
                .unwrap();
            if height >= h {
                return Ok(());
            }
        }
        Err(Error::InternalError(format!(
            "subscription terminated before we could reach target height of {}",
            h
        )))
    }

    pub async fn close(&mut self) -> Result<()> {
        self.send_cmd(DriverCommand::Terminate).await
    }

    async fn send_cmd(&mut self, cmd: DriverCommand) -> Result<()> {
        self.cmd_tx.send(cmd).map_err(|e| {
            Error::InternalError(format!(
                "WebSocket driver channel receiving end closed unexpectedly: {}",
                e.to_string()
            ))
        })
    }
}

type JsonResultTx = mpsc::UnboundedSender<Result<serde_json::Value>>;
type DriverCommandTx = mpsc::UnboundedSender<DriverCommand>;
type DriverCommandRx = mpsc::UnboundedReceiver<DriverCommand>;

#[derive(Debug, Clone)]
enum DriverCommand {
    Subscribe {
        id: String,
        query: String,
        subscription_tx: SubscriptionTx,
        response_tx: JsonResultTx,
    },
    Request {
        wrapper: serde_json::Value,
        response_tx: JsonResultTx,
    },
    Terminate,
}

pub struct ClientDriver {
    stream: WebSocketStream<ConnectStream>,
    cmd_rx: DriverCommandRx,
    subscribers: HashMap<String, HashMap<String, SubscriptionTx>>,
    pending_commands: HashMap<String, DriverCommand>,
}

impl ClientDriver {
    fn new(stream: WebSocketStream<ConnectStream>, cmd_rx: DriverCommandRx) -> Self {
        Self {
            stream,
            cmd_rx,
            subscribers: HashMap::new(),
            pending_commands: HashMap::new(),
        }
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some(res) = self.stream.next() => match res {
                    Ok(msg) => self.handle_incoming_msg(msg).await?,
                    Err(e) => return Err(
                        Error::WebSocketError(
                            format!("failed to read from WebSocket connection: {}", e),
                        ),
                    ),
                },
                Some(cmd) = self.cmd_rx.recv() => match cmd {
                    DriverCommand::Subscribe {
                        id,
                        query,
                        subscription_tx,
                        response_tx,
                    } => self.subscribe(id, query, subscription_tx, response_tx).await?,
                    DriverCommand::Request {
                        wrapper,
                        response_tx,
                    } => self.request(wrapper, response_tx).await?,
                    DriverCommand::Terminate => return self.close().await,
                },
            }
        }
    }

    async fn send_msg(&mut self, msg: Message) -> Result<()> {
        self.stream.send(msg).await.map_err(|e| {
            Error::WebSocketError(format!("failed to write to WebSocket connection: {}", e))
        })
    }

    async fn send_json(&mut self, req: &serde_json::Value) -> Result<()> {
        self.send_msg(Message::Text(serde_json::to_string(req).unwrap()))
            .await
    }

    async fn handle_incoming_msg(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Text(s) => self.handle_text_msg(s),
            Message::Ping(v) => self.pong(v).await,
            _ => Ok(()),
        }
    }

    fn handle_text_msg(&mut self, msg: String) -> Result<()> {
        debug!("Received incoming text message: {}", msg);

        let json_msg = serde_json::from_str::<serde_json::Value>(&msg)?;
        let id = json_msg.get("id").ok_or_else(|| {
            Error::MalformedResponse(format!("incoming message has no \"id\" field: {}", msg))
        })?;

        // Check if we recognize this ID as a response to an earlier request
        if id.is_string() {
            let id_str = id.as_str().unwrap();
            if self.pending_commands.contains_key(id_str) {
                return self.confirm_pending_command(id_str.to_string(), json_msg);
            }
        }

        // If we don't recognize the ID of the incoming message, it's most
        // likely an event.
        self.handle_event(json_msg)
    }

    fn handle_event(&mut self, ev: serde_json::Value) -> Result<()> {
        let result = match ev.get("result") {
            Some(p) => p,
            None => {
                error!("Failed to parse incoming message as an event: no \"result\" field");
                return Ok(());
            }
        };
        let query = match result.get("query").unwrap().as_str() {
            Some(q) => q.to_string(),
            None => {
                error!("Failed to parse incoming message as an event: cannot interpret \"query\" field as a string");
                return Ok(());
            }
        };
        self.publish_event(query, ev)
    }

    fn publish_event(&mut self, query: String, ev: serde_json::Value) -> Result<()> {
        let subs_for_query = match self.subscribers.get_mut(&query) {
            Some(s) => s,
            None => return Ok(()),
        };
        let mut disconnected = Vec::new();
        for (subs_id, subs_tx) in subs_for_query {
            if let Err(e) = subs_tx.send(Ok(ev.clone())) {
                warn!(
                    "Disconnecting subscription with ID {} due to channel send failure: {}",
                    subs_id, e
                );
                disconnected.push(subs_id.clone());
            }
        }
        let subs_for_query = self.subscribers.get_mut(&query).unwrap();
        for subs_id in disconnected {
            subs_for_query.remove(&subs_id);
        }
        Ok(())
    }

    fn confirm_pending_command(&mut self, id: String, wrapper: serde_json::Value) -> Result<()> {
        let pending_command = self.pending_commands.remove(&id).unwrap();
        match pending_command {
            DriverCommand::Subscribe {
                id,
                query,
                subscription_tx,
                response_tx,
            } => {
                self.confirm_pending_subscription(id, wrapper, query, subscription_tx, response_tx)
            }
            DriverCommand::Request {
                wrapper: outgoing_wrapper,
                response_tx,
            } => {
                let method = outgoing_wrapper
                    .get("method")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string();
                self.confirm_pending_request(method, wrapper, response_tx)
            }
            _ => panic!("Unexpected pending command type: {:?}", pending_command),
        }
    }

    fn confirm_pending_subscription(
        &mut self,
        id: String,
        wrapper: serde_json::Value,
        query: String,
        subscription_tx: SubscriptionTx,
        response_tx: JsonResultTx,
    ) -> Result<()> {
        if wrapper.get("error").is_some() {
            let _ = response_tx.send(Err(Error::Failed("subscribe".to_string(), wrapper)))?;
            return Ok(());
        }
        if wrapper.get("result").is_some() {
            return self.add_subscriber(id, query, subscription_tx, response_tx, wrapper);
        }
        error!("Missing result and error fields in wrapper response");
        Ok(())
    }

    fn add_subscriber(
        &mut self,
        id: String,
        query: String,
        subscription_tx: SubscriptionTx,
        response_tx: JsonResultTx,
        response: serde_json::Value,
    ) -> Result<()> {
        let subs_for_query = match self.subscribers.get_mut(&query) {
            Some(s) => s,
            None => {
                self.subscribers.insert(query.clone(), HashMap::new());
                self.subscribers.get_mut(&query).unwrap()
            }
        };
        subs_for_query.insert(id, subscription_tx);
        let _ = response_tx.send(Ok(response))?;
        Ok(())
    }

    fn confirm_pending_request(
        &mut self,
        method: String,
        wrapper: serde_json::Value,
        response_tx: JsonResultTx,
    ) -> Result<()> {
        if wrapper.get("error").is_some() {
            let _ = response_tx.send(Err(Error::Failed(method, wrapper)))?;
            return Ok(());
        }
        let _ = response_tx.send(Ok(wrapper))?;
        Ok(())
    }

    async fn pong(&mut self, v: Vec<u8>) -> Result<()> {
        self.send_msg(Message::Pong(v)).await
    }

    async fn subscribe(
        &mut self,
        id: String,
        query: String,
        subscription_tx: JsonResultTx,
        response_tx: JsonResultTx,
    ) -> Result<()> {
        // First check if we already have a subscription for this query
        if self.subscribers.contains_key(&query) {
            // There's no need to go to the effort of sending a subscription
            // request - confirm the subscription immediately
            return self.add_subscriber(
                id,
                query,
                subscription_tx,
                response_tx,
                // Send back an empty result object
                serde_json::Value::Object(serde_json::Map::new()),
            );
        }

        let req = json!({
            "id": id,
            "jsonrpc": "2.0",
            "method": "subscribe",
            "params": {
                "query": query,
            }
        });
        self.send_json(&req).await?;
        self.pending_commands.insert(
            id.clone(),
            DriverCommand::Subscribe {
                id,
                query,
                subscription_tx,
                response_tx,
            },
        );
        Ok(())
    }

    async fn request(
        &mut self,
        wrapper: serde_json::Value,
        response_tx: JsonResultTx,
    ) -> Result<()> {
        let id = wrapper.get("id").unwrap().as_str().unwrap().to_owned();
        self.send_json(&wrapper).await?;
        self.pending_commands.insert(
            id.clone(),
            DriverCommand::Request {
                wrapper,
                response_tx,
            },
        );
        Ok(())
    }

    async fn close(mut self) -> Result<()> {
        self.send_msg(Message::Close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: Cow::from("client closed WebSocket connection"),
        })))
        .await?;

        while let Some(res) = self.stream.next().await {
            if res.is_err() {
                return Ok(());
            }
        }
        Ok(())
    }
}
