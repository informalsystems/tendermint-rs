//! HTTP-based transport for Tendermint RPC Client, with WebSockets-based
//! subscription handling mechanism.
//!
//! The `client` and `http_ws` features are required to use this module.

use crate::client::subscription::{EventTx, PendingResultTx, SubscriptionState};
use crate::client::{Subscription, SubscriptionId, SubscriptionRouter};
use crate::endpoint::{subscribe, unsubscribe};
use crate::event::Event;
use crate::{request, response};
use crate::{Error, FullClient, MinimalClient, Request, Response, Result};
use async_trait::async_trait;
use async_tungstenite::tokio::{connect_async, TokioAdapter};
use async_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use async_tungstenite::tungstenite::protocol::CloseFrame;
use async_tungstenite::tungstenite::Message;
use async_tungstenite::WebSocketStream;
use bytes::buf::BufExt;
use futures::{SinkExt, StreamExt};
use hyper::header;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::convert::TryInto;
use tendermint::net;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

// We anticipate that this will be a relatively low-traffic command signaling
// mechanism (these commands are limited to subscribe/unsubscribe requests,
// which we assume won't occur very frequently).
const DEFAULT_WEBSOCKET_CMD_BUF_SIZE: usize = 20;

/// An HTTP-based Tendermint RPC client (a [`MinimalClient`] implementation).
/// Requires features `client` and `http_ws`.
///
/// Does not provide [`Event`] subscription facilities (see
/// [`HttpWebSocketClient`] for a client that does provide `Event` subscription
/// facilities).
///
/// ## Examples
///
/// We don't test this example automatically at present, but it has and can
/// been tested against a Tendermint node running on `localhost`.
///
/// ```ignore
/// use tendermint_rpc::{HttpClient, MinimalClient};
///
/// #[tokio::main]
/// async fn main() {
///     let client = HttpClient::new("tcp://127.0.0.1:26657".parse().unwrap())
///         .unwrap();
///
///     let abci_info = client.abci_info()
///         .await
///         .unwrap();
///
///     println!("Got ABCI info: {:?}", abci_info);
/// }
/// ```
///
/// [`MinimalClient`]: trait.MinimalClient.html
/// [`Event`]: ./event/struct.Event.html
/// [`HttpWebSocketClient`]: struct.HttpWebSocketClient.html
///
#[derive(Debug, Clone)]
pub struct HttpClient {
    host: String,
    port: u16,
}

#[async_trait]
impl MinimalClient for HttpClient {
    async fn perform<R>(&self, request: R) -> Result<R::Response>
    where
        R: Request,
    {
        http_request(&self.host, self.port, request).await
    }

    async fn close(self) -> Result<()> {
        Ok(())
    }
}

impl HttpClient {
    /// Create a new HTTP-based Tendermint RPC client.
    pub fn new(address: net::Address) -> Result<Self> {
        let (host, port) = get_tcp_host_port(address)?;
        Ok(HttpClient { host, port })
    }
}

/// An HTTP- and WebSocket-based Tendermint RPC client. Requires features
/// `client` and `http_ws`.
///
/// HTTP is used for all requests except those pertaining to [`Event`]
/// subscription. `Event` subscription is facilitated by a WebSocket
/// connection, which is opened as this client is created.
///
/// ## Examples
///
/// We don't test this example automatically at present, but it has and can
/// been tested against a Tendermint node running on `localhost`.
///
/// ```ignore
/// use tendermint_rpc::{HttpWebSocketClient, FullClient, MinimalClient};
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() {
///     let mut client = HttpWebSocketClient::new("tcp://127.0.0.1:26657".parse().unwrap())
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
///     while let Some(ev) = subs.next().await {
///         println!("Got event: {:?}", ev);
///         ev_count -= 1;
///         if ev_count < 0 {
///             break
///         }
///     }
///
///     client.unsubscribe(subs).await.unwrap();
///     client.close().await.unwrap();
/// }
/// ```
///
/// [`Event`]: ./event/struct.Event.html
///
#[derive(Debug)]
pub struct HttpWebSocketClient {
    host: String,
    port: u16,
    driver_handle: JoinHandle<Result<()>>,
    cmd_tx: mpsc::Sender<WebSocketDriverCmd>,
    next_subscription_id: SubscriptionId,
}

impl HttpWebSocketClient {
    /// Construct a full HTTP/WebSocket client directly.
    pub async fn new(address: net::Address) -> Result<Self> {
        let (host, port) = get_tcp_host_port(address)?;
        let (stream, _response) =
            connect_async(&format!("ws://{}:{}/websocket", &host, port)).await?;
        let (cmd_tx, cmd_rx) = mpsc::channel(DEFAULT_WEBSOCKET_CMD_BUF_SIZE);
        let driver = WebSocketSubscriptionDriver::new(stream, cmd_rx);
        let driver_handle = tokio::spawn(async move { driver.run().await });
        Ok(HttpWebSocketClient {
            host,
            port,
            driver_handle,
            cmd_tx,
            next_subscription_id: SubscriptionId::default(),
        })
    }

    /// In the absence of an `async` version of [`TryFrom`], this constructor
    /// provides an `async` way to upgrade an [`HttpClient`] to an
    /// [`HttpWebSocketClient`].
    ///
    /// [`TryFrom`]: https://doc.rust-lang.org/std/convert/trait.TryFrom.html
    /// [`HttpClient`]: struct.HttpClient.html
    /// [`HttpWebSocketClient`]: struct.HttpWebSocketClient.html
    ///
    pub async fn try_from(client: HttpClient) -> Result<HttpWebSocketClient> {
        HttpWebSocketClient::new(net::Address::Tcp {
            peer_id: None,
            host: client.host,
            port: client.port,
        })
        .await
    }

    async fn send_cmd(&mut self, cmd: WebSocketDriverCmd) -> Result<()> {
        self.cmd_tx.send(cmd).await.map_err(|e| {
            Error::internal_error(format!("failed to send command to client driver: {}", e))
        })
    }
}

#[async_trait]
impl MinimalClient for HttpWebSocketClient {
    async fn perform<R>(&self, request: R) -> Result<R::Response>
    where
        R: Request,
    {
        http_request(&self.host, self.port, request).await
    }

    /// Gracefully terminate the underlying connection.
    ///
    /// This sends a termination message to the client driver and blocks
    /// indefinitely until the driver terminates. If successfully closed, it
    /// returns the `Result` of the driver's async task.
    async fn close(mut self) -> Result<()> {
        self.send_cmd(WebSocketDriverCmd::Close).await?;
        self.driver_handle.await.map_err(|e| {
            Error::client_error(format!("failed to join client driver async task: {}", e))
        })?
    }
}

#[async_trait]
impl FullClient for HttpWebSocketClient {
    async fn subscribe_with_buf_size(
        &mut self,
        query: String,
        buf_size: usize,
    ) -> Result<Subscription> {
        let (event_tx, event_rx) = mpsc::channel(buf_size);
        let (result_tx, result_rx) = oneshot::channel();
        // We use the same ID for our subscription as for the JSONRPC request
        // so that we can correlate incoming RPC responses with specific
        // subscriptions. We use this to establish whether or not a
        // subscription request was successful, as well as whether we support
        // the remote endpoint's serialization format.
        let id = SubscriptionId::default();
        let req = request::Wrapper::new_with_id(
            id.clone().into(),
            subscribe::Request::new(query.clone()),
        );
        self.send_cmd(WebSocketDriverCmd::Subscribe {
            req,
            event_tx,
            result_tx,
        })
        .await?;
        // Wait to make sure our subscription request went through
        // successfully.
        result_rx.await.map_err(|e| {
            Error::client_error(format!(
                "failed to receive response from client driver for subscription request: {}",
                e
            ))
        })??;
        Ok(Subscription::new(id, query, event_rx))
    }

    async fn unsubscribe(&mut self, subscription: Subscription) -> Result<()> {
        let (result_tx, result_rx) = oneshot::channel();
        self.send_cmd(WebSocketDriverCmd::Unsubscribe {
            req: request::Wrapper::new(unsubscribe::Request::new(subscription.query.clone())),
            subscription,
            result_tx,
        })
        .await?;
        result_rx.await.map_err(|e| {
            Error::client_error(format!(
                "failed to receive response from client driver for unsubscribe request: {}",
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
    cmd_rx: mpsc::Receiver<WebSocketDriverCmd>,
}

impl WebSocketSubscriptionDriver {
    fn new(
        stream: WebSocketStream<TokioAdapter<TcpStream>>,
        cmd_rx: mpsc::Receiver<WebSocketDriverCmd>,
    ) -> Self {
        Self {
            stream,
            router: SubscriptionRouter::default(),
            cmd_rx,
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
                Some(cmd) = self.cmd_rx.next() => match cmd {
                    WebSocketDriverCmd::Subscribe {
                        req,
                        event_tx,
                        result_tx,
                    } => self.subscribe(req, event_tx, result_tx).await?,
                    WebSocketDriverCmd::Unsubscribe { req, subscription, result_tx } => {
                        self.unsubscribe(req, subscription, result_tx).await?
                    }
                    WebSocketDriverCmd::Close => return self.close().await,
                }
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
        req: request::Wrapper<subscribe::Request>,
        event_tx: EventTx,
        result_tx: PendingResultTx,
    ) -> Result<()> {
        // We require the outgoing request to have an ID that we can use as the
        // subscription ID.
        let subs_id = match req.id().clone().try_into() {
            Ok(id) => id,
            Err(e) => {
                let _ = result_tx.send(Err(e));
                return Ok(());
            }
        };
        if let Err(e) = self
            .send(Message::Text(serde_json::to_string_pretty(&req).unwrap()))
            .await
        {
            if result_tx.send(Err(e)).is_err() {
                return Err(Error::client_error(
                    "failed to respond internally to subscription request",
                ));
            }
            // One failure shouldn't bring down the entire client.
            return Ok(());
        }
        self.router
            .add_pending_subscribe(subs_id, req.params().query.clone(), event_tx, result_tx);
        Ok(())
    }

    async fn unsubscribe(
        &mut self,
        req: request::Wrapper<unsubscribe::Request>,
        subscription: Subscription,
        result_tx: PendingResultTx,
    ) -> Result<()> {
        if let Err(e) = self
            .send(Message::Text(serde_json::to_string_pretty(&req).unwrap()))
            .await
        {
            if result_tx.send(Err(e)).is_err() {
                return Err(Error::client_error(
                    "failed to respond internally to unsubscribe request",
                ));
            }
            return Ok(());
        }
        self.router.add_pending_unsubscribe(subscription, result_tx);
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
            Ok(_) => match self.router.subscription_state(&subs_id) {
                SubscriptionState::Pending => {
                    let _ = self.router.confirm_pending_subscribe(&subs_id);
                }
                SubscriptionState::Cancelling => {
                    let _ = self.router.confirm_pending_unsubscribe(&subs_id);
                }
                _ => (),
            },
            Err(e) => match self.router.subscription_state(&subs_id) {
                SubscriptionState::Pending => {
                    let _ = self.router.cancel_pending_subscribe(&subs_id, e);
                }
                SubscriptionState::Cancelling => {
                    let _ = self.router.cancel_pending_unsubscribe(&subs_id, e);
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
        req: request::Wrapper<subscribe::Request>,
        event_tx: mpsc::Sender<Result<Event>>,
        result_tx: oneshot::Sender<Result<()>>,
    },
    Unsubscribe {
        req: request::Wrapper<unsubscribe::Request>,
        subscription: Subscription,
        result_tx: oneshot::Sender<Result<()>>,
    },
    Close,
}

async fn http_request<R>(host: &str, port: u16, request: R) -> Result<R::Response>
where
    R: Request,
{
    let request_body = request.into_json();

    let mut request = hyper::Request::builder()
        .method("POST")
        .uri(&format!("http://{}:{}/", host, port))
        .body(hyper::Body::from(request_body.into_bytes()))?;

    {
        let headers = request.headers_mut();
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(
            header::USER_AGENT,
            format!("tendermint.rs/{}", env!("CARGO_PKG_VERSION"))
                .parse()
                .unwrap(),
        );
    }
    let http_client = hyper::Client::builder().build_http();
    let response = http_client.request(request).await?;
    let response_body = hyper::body::aggregate(response.into_body()).await?;
    R::Response::from_reader(response_body.reader())
}

fn get_tcp_host_port(address: net::Address) -> Result<(String, u16)> {
    match address {
        net::Address::Tcp { host, port, .. } => Ok((host, port)),
        other => Err(Error::invalid_params(&format!(
            "invalid RPC address: {:?}",
            other
        ))),
    }
}
