//! HTTP-based transport for Tendermint RPC Client, with WebSockets-based
//! subscription handling mechanism.

use async_trait::async_trait;
use async_tungstenite::{
    tokio::{connect_async, TokioAdapter},
    tungstenite::{
        protocol::{frame::coding::CloseCode, CloseFrame},
        Message,
    },
    WebSocketStream,
};
use bytes::buf::BufExt;
use futures::{stream::StreamExt, SinkExt};
use hyper::header;
use std::{borrow::Cow, io::Read};
use tendermint::net;
use tokio::{net::TcpStream, sync::mpsc, task::JoinHandle};

use crate::{
    client::transport::{EventConnection, EventProducer, SubscriptionTransport, Transport},
    endpoint::{subscribe, unsubscribe},
    event::Event,
    response::Response,
    Error, Request,
};

// We anticipate that this will be a relatively low-traffic command signaling
// mechanism (these commands are limited to subscribe/unsubscribe requests,
// which we assume won't occur very frequently).
const DEFAULT_WEBSOCKET_CMD_BUF_SIZE: usize = 50;

/// An HTTP-based transport layer for the Tendermint RPC Client. Subscriptions
/// are managed via a WebSockets connection which is maintained separately to
/// the HTTP request mechanisms.
#[derive(Debug)]
pub struct HttpWsTransport {
    host: String,
    port: u16,
}

#[derive(Debug)]
struct WsSubscriptionTransport {
    driver_hdl: JoinHandle<Result<(), Error>>,
    cmd_tx: mpsc::Sender<WsCmd>,
}

#[derive(Debug)]
struct WsDriver {
    stream: WebSocketStream<TokioAdapter<TcpStream>>,
    event_tx: mpsc::Sender<Event>,
    cmd_rx: mpsc::Receiver<WsCmd>,
}

enum WsCmd {
    Subscribe { query: String },
    Unsubscribe { query: String },
    Close,
}

#[async_trait]
impl Transport for HttpWsTransport {
    async fn request(&self, request_body: String) -> Result<String, Error> {
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
        let mut response_string = String::new();
        let _ = response_body
            .reader()
            .read_to_string(&mut response_string)
            .map_err(|e| Error::internal_error(format!("failed to read response body: {}", e)));
        Ok(response_string)
    }

    /// Initiates a new WebSocket connection to the remote endpoint.
    async fn new_event_connection(&self, event_buf_size: usize) -> Result<EventConnection, Error> {
        let (transport, event_producer) = WsSubscriptionTransport::connect(
            &format!("ws://{}:{}/websocket", self.host, self.port),
            event_buf_size,
            DEFAULT_WEBSOCKET_CMD_BUF_SIZE,
        )
        .await?;
        Ok(EventConnection::new(Box::new(transport), event_producer))
    }
}

impl HttpWsTransport {
    /// Create a new HTTP/WebSockets transport layer.
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
        Ok(HttpWsTransport { host, port })
    }
}

#[async_trait]
impl SubscriptionTransport for WsSubscriptionTransport {
    async fn subscribe(&mut self, query: String) -> Result<(), Error> {
        self.cmd_tx
            .send(WsCmd::Subscribe { query })
            .await
            .map_err(|e| {
                Error::internal_error(format!(
                    "failed to transmit subscription command to async WebSocket driver: {}",
                    e
                ))
            })
    }

    async fn unsubscribe(&mut self, query: String) -> Result<(), Error> {
        self.cmd_tx
            .send(WsCmd::Unsubscribe { query })
            .await
            .map_err(|e| {
                Error::internal_error(format!(
                    "failed to transmit unsubscribe command to async WebSocket driver: {}",
                    e
                ))
            })
    }

    async fn close(&mut self) -> Result<(), Error> {
        self.cmd_tx.send(WsCmd::Close).await.map_err(|e| {
            Error::internal_error(format!(
                "failed to send termination command to async task: {}",
                e
            ))
        })

        // TODO: Find a way to wait for the driver to terminate.
    }
}

impl WsSubscriptionTransport {
    async fn connect(
        url: &str,
        event_buf_size: usize,
        cmd_buf_size: usize,
    ) -> Result<(WsSubscriptionTransport, EventProducer), Error> {
        let (stream, _response) = connect_async(url).await?;
        let (event_tx, event_rx) = mpsc::channel(event_buf_size);
        let (cmd_tx, cmd_rx) = mpsc::channel(cmd_buf_size);
        let driver = WsDriver {
            stream,
            event_tx,
            cmd_rx,
        };
        let driver_hdl = tokio::spawn(async move { driver.run().await });
        Ok((
            WsSubscriptionTransport { driver_hdl, cmd_tx },
            EventProducer::new(event_rx),
        ))
    }
}

impl WsDriver {
    async fn run(mut self) -> Result<(), Error> {
        // TODO: Should this loop initiate a keepalive (ping) to the server on a regular basis?
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
                    WsCmd::Subscribe { query } => self.subscribe(query).await?,
                    WsCmd::Unsubscribe { query } => self.unsubscribe(query).await?,
                    WsCmd::Close => return self.close().await,
                }
            }
        }
    }

    async fn subscribe(&mut self, query: String) -> Result<(), Error> {
        let req = subscribe::Request::new(query);
        self.stream
            .send(Message::Text(req.into_json()))
            .await
            .map_err(|e| {
                Error::websocket_error(format!("failed to write to WebSocket connection: {}", e))
            })
    }

    async fn unsubscribe(&mut self, query: String) -> Result<(), Error> {
        let req = unsubscribe::Request::new(query);
        self.stream
            .send(Message::Text(req.into_json()))
            .await
            .map_err(|e| {
                Error::websocket_error(format!("failed to write to WebSocket connection: {}", e))
            })
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
            Ok(ev) => self.handle_event(ev).await,
            // TODO: Should we just ignore messages we can't deserialize?
            //       There are a number of possible messages we may receive
            //       from the WebSocket endpoint that we'll end up ignoring
            //       anyways (like responses for subscribe/unsubscribe
            //       requests).
            Err(_) => Ok(()),
        }
    }

    async fn handle_event(&mut self, ev: Event) -> Result<(), Error> {
        self.event_tx.send(ev).await.map_err(|e| {
            Error::internal_error(format!(
                "failed to publish incoming event to event producer: {}",
                e
            ))
        })
    }

    async fn pong(&mut self, v: Vec<u8>) -> Result<(), Error> {
        self.stream.send(Message::Pong(v)).await.map_err(|e| {
            Error::websocket_error(format!("failed to write WebSocket pong message: {}", e))
        })
    }

    async fn close(&mut self) -> Result<(), Error> {
        let _ = self
            .stream
            .send(Message::Close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: Cow::from("client closed WebSocket connection"),
            })))
            .await
            .map_err(|e| {
                Error::websocket_error(format!(
                    "failed to cleanly terminate WebSocket connection: {}",
                    e
                ))
            })?;

        while let Some(res) = self.stream.next().await {
            if let Err(_) = res {
                return Ok(());
            }
        }
        Ok(())
    }
}
