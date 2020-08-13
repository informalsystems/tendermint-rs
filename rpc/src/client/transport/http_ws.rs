//! HTTP-based transport for Tendermint RPC Client, with WebSockets-based
//! subscription handling mechanism.

use crate::client::subscription::{Subscription, SubscriptionId, SubscriptionRouter};
use crate::client::transport::{ClosableTransport, SubscriptionTransport, Transport};
use crate::endpoint::{subscribe, unsubscribe};
use crate::event::Event;
use crate::response::Response;
use crate::{Error, Method, Request};
use async_trait::async_trait;
use async_tungstenite::tokio::{connect_async, TokioAdapter};
use async_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use async_tungstenite::tungstenite::protocol::CloseFrame;
use async_tungstenite::tungstenite::Message;
use async_tungstenite::WebSocketStream;
use bytes::buf::BufExt;
use futures::{stream::StreamExt, SinkExt};
use hyper::header;
use std::borrow::Cow;
use tendermint::net;
use tokio::{net::TcpStream, sync::mpsc, task::JoinHandle};

// We anticipate that this will be a relatively low-traffic command signaling
// mechanism (these commands are limited to subscribe/unsubscribe requests,
// which we assume won't occur very frequently).
const DEFAULT_WEBSOCKET_CMD_BUF_SIZE: usize = 20;

/// An HTTP-based transport layer for the Tendermint RPC Client.
#[derive(Debug)]
pub struct HttpTransport {
    host: String,
    port: u16,
}

#[async_trait]
impl Transport for HttpTransport {
    type SubscriptionTransport = WebSocketSubscriptionTransport;

    async fn request<R>(&self, request: R) -> Result<R::Response, Error>
    where
        R: Request,
    {
        // TODO(thane): Find a way to enforce this at compile time.
        match request.method() {
            Method::Subscribe | Method::Unsubscribe => return Err(
                Error::internal_error(
                    "HttpTransport does not support subscribe/unsubscribe methods - rather use WebSocketSubscriptionTransport"
                )
            ),
            _ => (),
        }

        let request_body = request.into_json();

        let mut request = hyper::Request::builder()
            .method("POST")
            .uri(&format!("http://{}:{}/", self.host, self.port))
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

    async fn subscription_transport(&self) -> Result<Self::SubscriptionTransport, Error> {
        WebSocketSubscriptionTransport::new(&format!("ws://{}:{}/websocket", self.host, self.port))
            .await
    }
}

#[async_trait]
impl ClosableTransport for HttpTransport {
    async fn close(self) -> Result<(), Error> {
        Ok(())
    }
}

impl HttpTransport {
    /// Create a new HTTP transport layer.
    pub fn new(address: net::Address) -> Result<Self, Error> {
        let (host, port) = match address {
            net::Address::Tcp { host, port, .. } => (host, port),
            other => {
                return Err(Error::invalid_params(&format!(
                    "invalid RPC address: {:?}",
                    other
                )))
            }
        };
        Ok(HttpTransport { host, port })
    }
}

/// A WebSocket-based transport for interacting with the Tendermint RPC
/// subscriptions interface.
#[derive(Debug)]
pub struct WebSocketSubscriptionTransport {
    driver_hdl: JoinHandle<Result<(), Error>>,
    cmd_tx: mpsc::Sender<WebSocketDriverCmd>,
    next_subs_id: SubscriptionId,
}

impl WebSocketSubscriptionTransport {
    async fn new(url: &str) -> Result<Self, Error> {
        let (stream, _response) = connect_async(url).await?;
        let (cmd_tx, cmd_rx) = mpsc::channel(DEFAULT_WEBSOCKET_CMD_BUF_SIZE);
        let driver = WebSocketSubscriptionDriver::new(stream, cmd_rx);
        let driver_hdl = tokio::spawn(async move { driver.run().await });
        Ok(Self {
            driver_hdl,
            cmd_tx,
            next_subs_id: SubscriptionId::from(0),
        })
    }

    async fn send_cmd(&mut self, cmd: WebSocketDriverCmd) -> Result<(), Error> {
        self.cmd_tx.send(cmd).await.map_err(|e| {
            Error::internal_error(format!(
                "failed to transmit command to async WebSocket driver: {}",
                e
            ))
        })
    }

    fn next_subscription_id(&mut self) -> SubscriptionId {
        let res = self.next_subs_id.clone();
        self.next_subs_id = self.next_subs_id.next();
        res
    }
}

#[async_trait]
impl SubscriptionTransport for WebSocketSubscriptionTransport {
    async fn subscribe(
        &mut self,
        request: subscribe::Request,
        event_tx: mpsc::Sender<Event>,
    ) -> Result<SubscriptionId, Error> {
        let id = self.next_subscription_id();
        self.send_cmd(WebSocketDriverCmd::Subscribe {
            request,
            id: id.clone(),
            event_tx,
        })
        .await?;
        Ok(id)
    }

    async fn unsubscribe(
        &mut self,
        request: unsubscribe::Request,
        subscription: Subscription,
    ) -> Result<(), Error> {
        self.send_cmd(WebSocketDriverCmd::Unsubscribe {
            request,
            subscription,
        })
        .await
    }
}

#[async_trait]
impl ClosableTransport for WebSocketSubscriptionTransport {
    async fn close(mut self) -> Result<(), Error> {
        self.send_cmd(WebSocketDriverCmd::Close).await?;
        self.driver_hdl.await.map_err(|e| {
            Error::internal_error(format!("failed to join async WebSocket driver task: {}", e))
        })?
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

    async fn run(mut self) -> Result<(), Error> {
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
                    WebSocketDriverCmd::Subscribe { request, id, event_tx } => self.subscribe(request, id, event_tx).await?,
                    WebSocketDriverCmd::Unsubscribe { request, subscription } => self.unsubscribe(request, subscription).await?,
                    WebSocketDriverCmd::Close => return self.close().await,
                }
            }
        }
    }

    async fn send(&mut self, msg: Message) -> Result<(), Error> {
        self.stream.send(msg).await.map_err(|e| {
            Error::websocket_error(format!("failed to write to WebSocket connection: {}", e))
        })
    }

    async fn subscribe(
        &mut self,
        request: subscribe::Request,
        id: SubscriptionId,
        event_tx: mpsc::Sender<Event>,
    ) -> Result<(), Error> {
        self.send(Message::Text(request.clone().into_json()))
            .await?;
        self.router.add(id, request.query, event_tx);
        Ok(())
    }

    async fn unsubscribe(
        &mut self,
        request: unsubscribe::Request,
        subs: Subscription,
    ) -> Result<(), Error> {
        self.send(Message::Text(request.clone().into_json()))
            .await?;
        self.router.remove(subs);
        Ok(())
    }

    async fn handle_incoming_msg(&mut self, msg: Message) -> Result<(), Error> {
        match msg {
            Message::Text(s) => self.handle_text_msg(s).await,
            Message::Ping(v) => self.pong(v).await,
            Message::Pong(_) | Message::Binary(_) => Ok(()),
            Message::Close(_) => Ok(()),
        }
    }

    async fn handle_text_msg(&mut self, msg: String) -> Result<(), Error> {
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

    async fn pong(&mut self, v: Vec<u8>) -> Result<(), Error> {
        self.send(Message::Pong(v)).await
    }

    async fn close(mut self) -> Result<(), Error> {
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
        request: subscribe::Request,
        id: SubscriptionId,
        event_tx: mpsc::Sender<Event>,
    },
    Unsubscribe {
        request: unsubscribe::Request,
        subscription: Subscription,
    },
    Close,
}
