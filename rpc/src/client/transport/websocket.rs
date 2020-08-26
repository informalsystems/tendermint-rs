//! WebSocket-based clients for accessing Tendermint RPC functionality.

use crate::client::subscription::{SubscriptionState, TerminateSubscription};
use crate::client::sync::{bounded, unbounded, ChannelRx, ChannelTx};
use crate::client::transport::get_tcp_host_port;
use crate::client::{ClosableClient, SubscriptionRouter};
use crate::endpoint::{subscribe, unsubscribe};
use crate::event::Event;
use crate::request::Wrapper;
use crate::{response, Error, Response, Result, Subscription, SubscriptionClient, SubscriptionId};
use async_trait::async_trait;
use async_tungstenite::tokio::{connect_async, TokioAdapter};
use async_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use async_tungstenite::tungstenite::protocol::CloseFrame;
use async_tungstenite::tungstenite::Message;
use async_tungstenite::WebSocketStream;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::convert::TryInto;
use tendermint::net;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;

/// Tendermint RPC client that provides [`Event`] subscription capabilities
/// over JSON-RPC over a WebSocket connection.
///
/// In order to not block the calling task, this client spawns an asynchronous
/// driver that continuously interacts with the actual WebSocket connection.
/// The `WebSocketSubscriptionClient` itself is effectively just a handle to
/// this driver. This driver is spawned as the client is created.
///
/// To terminate the client and the driver, simply use its [`close`] method.
///
/// ## Examples
///
/// ```rust,ignore
/// use tendermint_rpc::{WebSocketSubscriptionClient, SubscriptionClient, ClosableClient};
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() {
///     let mut client = WebSocketSubscriptionClient::new("tcp://127.0.0.1:26657".parse().unwrap())
///         .await
///         .unwrap();
///
///     let mut subs = client.subscribe("tm.event='NewBlock'".to_string())
///         .await
///         .unwrap();
///
///     // Grab 5 NewBlock events
///     let mut ev_count = 5_i32;
///
///     while let Some(res) = subs.next().await {
///         let ev = res.unwrap();
///         println!("Got event: {:?}", ev);
///         ev_count -= 1;
///         if ev_count < 0 {
///             break
///         }
///     }
///
///     // Sends an unsubscribe request via the WebSocket connection, but keeps
///     // the connection open.
///     subs.terminate().await.unwrap();
///
///     // Attempt to gracefully terminate the WebSocket connection.
///     client.close().await.unwrap();
/// }
/// ```
///
/// [`Event`]: ./event/struct.Event.html
/// [`close`]: struct.WebSocketSubscriptionClient.html#method.close
#[derive(Debug)]
pub struct WebSocketSubscriptionClient {
    host: String,
    port: u16,
    driver_handle: JoinHandle<Result<()>>,
    cmd_tx: ChannelTx<WebSocketDriverCmd>,
    terminate_tx: ChannelTx<TerminateSubscription>,
}

impl WebSocketSubscriptionClient {
    /// Construct a WebSocket client. Immediately attempts to open a WebSocket
    /// connection to the node with the given address.
    pub async fn new(address: net::Address) -> Result<Self> {
        let (host, port) = get_tcp_host_port(address)?;
        let (stream, _response) =
            connect_async(&format!("ws://{}:{}/websocket", &host, port)).await?;
        let (cmd_tx, cmd_rx) = unbounded();
        let (terminate_tx, terminate_rx) = unbounded();
        let driver = WebSocketSubscriptionDriver::new(stream, cmd_rx, terminate_rx);
        let driver_handle = tokio::spawn(async move { driver.run().await });
        Ok(Self {
            host,
            port,
            driver_handle,
            cmd_tx,
            terminate_tx,
        })
    }

    async fn send_cmd(&mut self, cmd: WebSocketDriverCmd) -> Result<()> {
        self.cmd_tx.send(cmd).await.map_err(|e| {
            Error::client_internal_error(format!("failed to send command to client driver: {}", e))
        })
    }
}

#[async_trait]
impl SubscriptionClient for WebSocketSubscriptionClient {
    async fn subscribe_with_buf_size(
        &mut self,
        query: String,
        buf_size: usize,
    ) -> Result<Subscription> {
        let (event_tx, event_rx) = if buf_size == 0 {
            unbounded()
        } else {
            bounded(buf_size)
        };
        let (result_tx, mut result_rx) = unbounded::<Result<()>>();
        // NB: We assume here that the wrapper generates a unique ID for our
        // subscription.
        let req = Wrapper::new(subscribe::Request::new(query.clone()));
        let id: SubscriptionId = req.id().clone().try_into()?;
        self.send_cmd(WebSocketDriverCmd::Subscribe {
            req,
            event_tx,
            result_tx,
        })
        .await?;
        // Wait to make sure our subscription request went through
        // successfully.
        result_rx.recv().await.ok_or_else(|| {
            Error::client_internal_error("failed to hear back from WebSocket driver".to_string())
        })??;
        Ok(Subscription::new(
            id,
            query,
            event_rx,
            self.terminate_tx.clone(),
        ))
    }
}

#[async_trait]
impl ClosableClient for WebSocketSubscriptionClient {
    /// Attempt to gracefully close the WebSocket connection.
    async fn close(mut self) -> Result<()> {
        self.cmd_tx.send(WebSocketDriverCmd::Close).await?;
        self.driver_handle.await.map_err(|e| {
            Error::client_internal_error(format!(
                "failed while waiting for WebSocket driver task to terminate: {}",
                e
            ))
        })?
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GenericJSONResponse(serde_json::Value);

impl Response for GenericJSONResponse {}

#[derive(Debug)]
struct WebSocketSubscriptionDriver {
    stream: WebSocketStream<TokioAdapter<TcpStream>>,
    router: SubscriptionRouter,
    cmd_rx: ChannelRx<WebSocketDriverCmd>,
    terminate_rx: ChannelRx<TerminateSubscription>,
}

impl WebSocketSubscriptionDriver {
    fn new(
        stream: WebSocketStream<TokioAdapter<TcpStream>>,
        cmd_rx: ChannelRx<WebSocketDriverCmd>,
        terminate_rx: ChannelRx<TerminateSubscription>,
    ) -> Self {
        Self {
            stream,
            router: SubscriptionRouter::default(),
            cmd_rx,
            terminate_rx,
        }
    }

    async fn run(mut self) -> Result<()> {
        // TODO(thane): Should this loop initiate a keepalive (ping) to the
        //              server on a regular basis?
        loop {
            tokio::select! {
                Some(res) = self.stream.next() => match res {
                    Ok(msg) => self.handle_incoming_msg(msg).await?,
                    Err(e) => return Err(
                        Error::websocket_error(
                            format!("failed to read from WebSocket connection: {}", e),
                        ),
                    ),
                },
                Some(cmd) = self.cmd_rx.recv() => match cmd {
                    WebSocketDriverCmd::Subscribe {
                        req,
                        event_tx,
                        result_tx,
                    } => self.subscribe(req, event_tx, result_tx).await?,
                    WebSocketDriverCmd::Close => return self.close().await,
                },
                Some(term) = self.terminate_rx.recv() => self.unsubscribe(term).await?,
            }
        }
    }

    async fn send(&mut self, msg: Message) -> Result<()> {
        self.stream.send(msg).await.map_err(|e| {
            Error::websocket_error(format!("failed to write to WebSocket connection: {}", e))
        })
    }

    async fn subscribe(
        &mut self,
        req: Wrapper<subscribe::Request>,
        event_tx: ChannelTx<Result<Event>>,
        mut result_tx: ChannelTx<Result<()>>,
    ) -> Result<()> {
        let id: SubscriptionId = req.id().clone().try_into()?;
        let query = req.params().query.clone();
        if let Err(e) = self
            .send(Message::Text(serde_json::to_string_pretty(&req).unwrap()))
            .await
        {
            let _ = result_tx.send(Err(e)).await;
            return Ok(());
        }
        self.router
            .pending_add(id.as_ref(), &id, query, event_tx, result_tx);
        Ok(())
    }

    async fn unsubscribe(&mut self, mut term: TerminateSubscription) -> Result<()> {
        let req = Wrapper::new(unsubscribe::Request::new(term.query.clone()));
        let id: SubscriptionId = req.id().clone().try_into()?;
        if let Err(e) = self
            .send(Message::Text(serde_json::to_string_pretty(&req).unwrap()))
            .await
        {
            let _ = term.result_tx.send(Err(e)).await;
            return Ok(());
        }
        self.router
            .pending_remove(id.as_ref(), &id, term.query.clone(), term.result_tx);
        Ok(())
    }

    async fn handle_incoming_msg(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Text(s) => self.handle_text_msg(s).await,
            Message::Ping(v) => self.pong(v).await,
            Message::Pong(_) | Message::Binary(_) => Ok(()),
            Message::Close(_) => Ok(()),
        }
    }

    async fn handle_text_msg(&mut self, msg: String) -> Result<()> {
        match Event::from_string(&msg) {
            Ok(ev) => {
                self.router.publish(ev).await;
                Ok(())
            }
            Err(_) => match serde_json::from_str::<response::Wrapper<GenericJSONResponse>>(&msg) {
                Ok(wrapper) => self.handle_generic_response(wrapper).await,
                _ => Ok(()),
            },
        }
    }

    async fn handle_generic_response(
        &mut self,
        wrapper: response::Wrapper<GenericJSONResponse>,
    ) -> Result<()> {
        let subs_id: SubscriptionId = match wrapper.id().clone().try_into() {
            Ok(id) => id,
            // Just ignore the message if it doesn't have an intelligible ID.
            Err(_) => return Ok(()),
        };
        match wrapper.into_result() {
            Ok(_) => match self.router.subscription_state(subs_id.as_ref()) {
                SubscriptionState::Pending => {
                    let _ = self.router.confirm_add(subs_id.as_ref()).await;
                }
                SubscriptionState::Cancelling => {
                    let _ = self.router.confirm_remove(subs_id.as_ref()).await;
                }
                SubscriptionState::Active => {
                    if let Some(event_tx) = self.router.get_active_subscription_mut(&subs_id) {
                        let _ = event_tx.send(
                            Err(Error::websocket_error(
                                "failed to parse incoming response from remote WebSocket endpoint - does this client support the remote's RPC version?",
                            )),
                        ).await;
                    }
                }
                SubscriptionState::NotFound => (),
            },
            Err(e) => match self.router.subscription_state(subs_id.as_ref()) {
                SubscriptionState::Pending => {
                    let _ = self.router.cancel_add(subs_id.as_ref(), e).await;
                }
                SubscriptionState::Cancelling => {
                    let _ = self.router.cancel_remove(subs_id.as_ref(), e).await;
                }
                // This is important to allow the remote endpoint to
                // arbitrarily send error responses back to specific
                // subscriptions.
                SubscriptionState::Active => {
                    if let Some(event_tx) = self.router.get_active_subscription_mut(&subs_id) {
                        // TODO(thane): Does an error here warrant terminating the subscription, or the driver?
                        let _ = event_tx.send(Err(e)).await;
                    }
                }
                SubscriptionState::NotFound => (),
            },
        }

        Ok(())
    }

    async fn pong(&mut self, v: Vec<u8>) -> Result<()> {
        self.send(Message::Pong(v)).await
    }

    async fn close(mut self) -> Result<()> {
        self.send(Message::Close(Some(CloseFrame {
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

#[derive(Debug)]
enum WebSocketDriverCmd {
    Subscribe {
        req: Wrapper<subscribe::Request>,
        event_tx: ChannelTx<Result<Event>>,
        result_tx: ChannelTx<Result<()>>,
    },
    Close,
}
