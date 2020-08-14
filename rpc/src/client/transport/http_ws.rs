//! HTTP-based transport for Tendermint RPC Client, with WebSockets-based
//! subscription handling mechanism.
//!
//! The `client` and `http_ws` features are required to use this module.

use crate::client::{Subscription, SubscriptionId, SubscriptionRouter};
use crate::endpoint::{subscribe, unsubscribe};
use crate::event::Event;
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
use std::borrow::Cow;
use tendermint::net;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

// We anticipate that this will be a relatively low-traffic command signaling
// mechanism (these commands are limited to subscribe/unsubscribe requests,
// which we assume won't occur very frequently).
const DEFAULT_WEBSOCKET_CMD_BUF_SIZE: usize = 20;

/// An HTTP-based Tendermint RPC client (a [`MinimalClient`] implementation).
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

/// An HTTP- and WebSocket-based Tendermint RPC client.
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

    /// In the absence of an `async` version of [`std::convert::TryFrom`],
    /// this constructor provides an `async` way to upgrade an [`HttpClient`]
    /// to an [`HttpWebSocketClient`].
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
            Error::internal_error(format!("failed to join client driver async task: {}", e))
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
        let (response_tx, response_rx) = oneshot::channel();
        let id = self.next_subscription_id.advance();
        self.send_cmd(WebSocketDriverCmd::Subscribe {
            id: id.clone(),
            query: query.clone(),
            event_tx,
            response_tx,
        })
        .await?;
        // Wait to make sure our subscription request went through
        // successfully.
        response_rx.await.map_err(|e| {
            Error::internal_error(format!(
                "failed to receive response from client driver for subscription request: {}",
                e
            ))
        })??;
        Ok(Subscription::new(id, query, event_rx))
    }

    async fn unsubscribe(&mut self, subscription: Subscription) -> Result<()> {
        // TODO(thane): Should we insist on a response here to ensure the
        //              subscription was actually terminated? Right now this is
        //              just fire-and-forget.
        self.send_cmd(WebSocketDriverCmd::Unsubscribe(subscription))
            .await
    }
}

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
            router: SubscriptionRouter::new(),
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
                        id,
                        query,
                        event_tx,
                        response_tx,
                    } => self.subscribe(id, query, event_tx, response_tx).await?,
                    WebSocketDriverCmd::Unsubscribe(subscription) => self.unsubscribe(subscription).await?,
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
        id: SubscriptionId,
        query: String,
        event_tx: mpsc::Sender<Event>,
        response_tx: oneshot::Sender<Result<()>>,
    ) -> Result<()> {
        if let Err(e) = self
            .send(Message::Text(
                subscribe::Request::new(query.clone()).into_json(),
            ))
            .await
        {
            if response_tx.send(Err(e)).is_err() {
                return Err(Error::internal_error(
                    "failed to respond internally to subscription request",
                ));
            }
            // One failure shouldn't bring down the entire client.
            return Ok(());
        }
        // TODO(thane): Should we wait for a response from the remote endpoint?
        self.router.add(id, query, event_tx);
        // TODO(thane): How do we deal with the case where the following
        //              response fails?
        if response_tx.send(Ok(())).is_err() {
            return Err(Error::internal_error(
                "failed to respond internally to subscription request",
            ));
        }
        Ok(())
    }

    async fn unsubscribe(&mut self, subs: Subscription) -> Result<()> {
        self.send(Message::Text(
            unsubscribe::Request::new(subs.query.clone()).into_json(),
        ))
        .await?;
        self.router.remove(subs);
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
        match Event::from_string(msg) {
            Ok(ev) => {
                self.router.publish(ev).await;
                Ok(())
            }
            // TODO(thane): Should we just ignore messages we can't
            //              deserialize? There are a number of possible
            //              messages we may receive from the WebSocket endpoint
            //              that we'll end up ignoring anyways (like responses
            //              for subscribe/unsubscribe requests).
            Err(_) => Ok(()),
        }
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
        id: SubscriptionId,
        query: String,
        event_tx: mpsc::Sender<Event>,
        response_tx: oneshot::Sender<Result<()>>,
    },
    Unsubscribe(Subscription),
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
