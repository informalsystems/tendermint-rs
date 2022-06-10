//! WebSocket-based clients for accessing Tendermint RPC functionality.

use core::convert::TryInto;

use async_trait::async_trait;

use crate::client::transport::websocket::{plumbing, WebSocketClientUrl, WebSocketConfig};
use crate::v0_34::client::subscription::{Subscription, SubscriptionClient};
use crate::v0_34::event::Event;
use crate::{error::Error, prelude::*, query::Query, Client, Request, SimpleRequest, Url};

/// Tendermint RPC client that provides access to all RPC functionality
/// (including [`Event`] subscription) over a WebSocket connection.
///
/// This type provides an RPC client compatible with Tendermint RPC version 0.34.
///
/// The `WebSocketClient` itself is effectively just a handle to its driver
/// The driver is the component of the client that actually interacts with the
/// remote RPC over the WebSocket connection. The `WebSocketClient` can
/// therefore be cloned into different asynchronous contexts, effectively
/// allowing for asynchronous access to the driver.
///
/// It is the caller's responsibility to spawn an asynchronous task in which to
/// execute the [`WebSocketClientDriver::run`] method. See the example below.
///
/// Dropping [`Subscription`]s will automatically terminate them (the
/// `WebSocketClientDriver` detects a disconnected channel and removes the
/// subscription from its internal routing table). When all subscriptions to a
/// particular query have disconnected, the driver will automatically issue an
/// unsubscribe request to the remote RPC endpoint.
///
/// ### Timeouts
///
/// The WebSocket client connection times out after 30 seconds if it does not
/// receive anything at all from the server. This will automatically return
/// errors to all active subscriptions and terminate them.
///
/// This is not configurable at present.
///
/// ### Keep-Alive
///
/// The WebSocket client implements a keep-alive mechanism whereby it sends a
/// PING message to the server every 27 seconds, matching the PING cadence of
/// the Tendermint server (see [this code][tendermint-websocket-ping] for
/// details).
///
/// This is not configurable at present.
///
/// ## Examples
///
/// ```rust,ignore
/// use tendermint::abci::Transaction;
/// use tendermint_rpc::{WebSocketClient, SubscriptionClient, Client};
/// use tendermint_rpc::query::EventType;
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() {
///     let (client, driver) = WebSocketClient::new("ws://127.0.0.1:26657/websocket")
///         .await
///         .unwrap();
///     let driver_handle = tokio::spawn(async move { driver.run().await });
///
///     // Standard client functionality
///     let tx = format!("some-key=some-value");
///     client.broadcast_tx_async(Transaction::from(tx.into_bytes())).await.unwrap();
///
///     // Subscription functionality
///     let mut subs = client.subscribe(EventType::NewBlock.into())
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
///             break;
///         }
///     }
///
///     // Signal to the driver to terminate.
///     client.close().unwrap();
///     // Await the driver's termination to ensure proper connection closure.
///     let _ = driver_handle.await.unwrap();
/// }
/// ```
///
/// [tendermint-websocket-ping]: https://github.com/tendermint/tendermint/blob/309e29c245a01825fc9630103311fd04de99fa5e/rpc/jsonrpc/server/ws_handler.go#L28
#[derive(Debug, Clone)]
pub struct WebSocketClient {
    inner: plumbing::Client<Event>,
}

impl WebSocketClient {
    /// Construct a new WebSocket-based client connecting to the given
    /// Tendermint node's RPC endpoint.
    ///
    /// Supports both `ws://` and `wss://` protocols.
    pub async fn new<U>(url: U) -> Result<(Self, WebSocketClientDriver), Error>
    where
        U: TryInto<WebSocketClientUrl, Error = Error>,
    {
        Self::new_with_config(url, None).await
    }

    /// Construct a new WebSocket-based client connecting to the given
    /// Tendermint node's RPC endpoint.
    ///
    /// Supports both `ws://` and `wss://` protocols.
    pub async fn new_with_config<U>(
        url: U,
        config: Option<WebSocketConfig>,
    ) -> Result<(Self, WebSocketClientDriver), Error>
    where
        U: TryInto<WebSocketClientUrl, Error = Error>,
    {
        let url = url.try_into()?;
        let url: Url = url.into();

        let (inner, driver) = if url.is_secure() {
            plumbing::Client::new_secure(url, config).await?
        } else {
            plumbing::Client::new_unsecure(url, config).await?
        };

        Ok((Self { inner }, WebSocketClientDriver { inner: driver }))
    }
}

#[async_trait]
impl Client for WebSocketClient {
    async fn perform<R>(&self, request: R) -> Result<<R as Request>::Response, Error>
    where
        R: SimpleRequest,
    {
        self.inner.perform(request).await
    }
}

#[async_trait]
impl SubscriptionClient for WebSocketClient {
    async fn subscribe(&self, query: Query) -> Result<Subscription, Error> {
        let subscription = self.inner.subscribe(query).await?;
        Ok(subscription.into())
    }

    async fn unsubscribe(&self, query: Query) -> Result<(), Error> {
        self.inner.unsubscribe(query).await
    }

    fn close(self) -> Result<(), Error> {
        self.inner.close()
    }
}

/// Drives the WebSocket connection for a `WebSocketClient` instance.
///
/// This is the primary component responsible for transport-level interaction
/// with the remote WebSocket endpoint.
pub struct WebSocketClientDriver {
    inner: plumbing::Driver<Event>,
}

impl WebSocketClientDriver {
    /// Executes the WebSocket driver, which manages the underlying WebSocket
    /// transport.
    pub async fn run(self) -> Result<(), Error> {
        self.inner.run().await
    }
}

#[cfg(test)]
mod test {
    use alloc::collections::BTreeMap as HashMap;
    use core::str::FromStr;
    use std::{path::PathBuf, println};

    use async_tungstenite::{
        tokio::{accept_async, TokioAdapter},
        tungstenite::{
            protocol::{frame::coding::CloseCode, CloseFrame},
            Message,
        },
        WebSocketStream,
    };
    use futures::prelude::*;
    use tendermint_config::net;
    use tokio::{
        fs,
        net::{TcpListener, TcpStream},
        task::JoinHandle,
    };

    use super::*;
    use crate::{
        client::sync::{unbounded, ChannelRx, ChannelTx},
        endpoint::{subscribe, unsubscribe},
        query::EventType,
        request, response, Id, Method, Response,
    };

    // Interface to a driver that manages all incoming WebSocket connections.
    struct TestServer {
        node_addr: net::Address,
        driver_hdl: JoinHandle<Result<(), Error>>,
        terminate_tx: ChannelTx<Result<(), Error>>,
        event_tx: ChannelTx<Event>,
    }

    impl TestServer {
        async fn new(addr: &str) -> Self {
            let listener = TcpListener::bind(addr).await.unwrap();
            let local_addr = listener.local_addr().unwrap();
            let node_addr = net::Address::Tcp {
                peer_id: None,
                host: local_addr.ip().to_string(),
                port: local_addr.port(),
            };
            let (terminate_tx, terminate_rx) = unbounded();
            let (event_tx, event_rx) = unbounded();
            let driver = TestServerDriver::new(listener, event_rx, terminate_rx);
            let driver_hdl = tokio::spawn(async move { driver.run().await });
            Self {
                node_addr,
                driver_hdl,
                terminate_tx,
                event_tx,
            }
        }

        fn publish_event(&mut self, ev: Event) -> Result<(), Error> {
            self.event_tx.send(ev)
        }

        async fn terminate(self) -> Result<(), Error> {
            self.terminate_tx.send(Ok(())).unwrap();
            self.driver_hdl.await.unwrap()
        }
    }

    // Manages all incoming WebSocket connections.
    struct TestServerDriver {
        listener: TcpListener,
        event_rx: ChannelRx<Event>,
        terminate_rx: ChannelRx<Result<(), Error>>,
        handlers: Vec<TestServerHandler>,
    }

    impl TestServerDriver {
        fn new(
            listener: TcpListener,
            event_rx: ChannelRx<Event>,
            terminate_rx: ChannelRx<Result<(), Error>>,
        ) -> Self {
            Self {
                listener,
                event_rx,
                terminate_rx,
                handlers: Vec::new(),
            }
        }

        async fn run(mut self) -> Result<(), Error> {
            loop {
                tokio::select! {
                    Some(ev) = self.event_rx.recv() => self.publish_event(ev),
                    res = self.listener.accept() => {
                        let (stream, _) = res.unwrap();
                        self.handle_incoming(stream).await
                    }
                    Some(res) = self.terminate_rx.recv() => {
                        self.terminate().await;
                        return res;
                    },
                }
            }
        }

        // Publishes the given event to all subscribers for the query relating
        // to the event.
        fn publish_event(&mut self, ev: Event) {
            for handler in &mut self.handlers {
                handler.publish_event(ev.clone());
            }
        }

        async fn handle_incoming(&mut self, stream: TcpStream) {
            self.handlers.push(TestServerHandler::new(stream).await);
        }

        async fn terminate(&mut self) {
            while !self.handlers.is_empty() {
                let handler = match self.handlers.pop() {
                    Some(h) => h,
                    None => break,
                };
                let _ = handler.terminate().await;
            }
        }
    }

    // Interface to a driver that manages a single incoming WebSocket
    // connection.
    struct TestServerHandler {
        driver_hdl: JoinHandle<Result<(), Error>>,
        terminate_tx: ChannelTx<Result<(), Error>>,
        event_tx: ChannelTx<Event>,
    }

    impl TestServerHandler {
        async fn new(stream: TcpStream) -> Self {
            let conn: WebSocketStream<TokioAdapter<TcpStream>> =
                accept_async(stream).await.unwrap();
            let (terminate_tx, terminate_rx) = unbounded();
            let (event_tx, event_rx) = unbounded();
            let driver = TestServerHandlerDriver::new(conn, event_rx, terminate_rx);
            let driver_hdl = tokio::spawn(async move { driver.run().await });
            Self {
                driver_hdl,
                terminate_tx,
                event_tx,
            }
        }

        fn publish_event(&mut self, ev: Event) {
            let _ = self.event_tx.send(ev);
        }

        async fn terminate(self) -> Result<(), Error> {
            self.terminate_tx.send(Ok(()))?;
            self.driver_hdl.await.unwrap()
        }
    }

    // Manages interaction with a single incoming WebSocket connection.
    struct TestServerHandlerDriver {
        conn: WebSocketStream<TokioAdapter<TcpStream>>,
        event_rx: ChannelRx<Event>,
        terminate_rx: ChannelRx<Result<(), Error>>,
        // A mapping of subscription queries to subscription IDs for this
        // connection.
        subscriptions: HashMap<String, String>,
    }

    impl TestServerHandlerDriver {
        fn new(
            conn: WebSocketStream<TokioAdapter<TcpStream>>,
            event_rx: ChannelRx<Event>,
            terminate_rx: ChannelRx<Result<(), Error>>,
        ) -> Self {
            Self {
                conn,
                event_rx,
                terminate_rx,
                subscriptions: HashMap::new(),
            }
        }

        async fn run(mut self) -> Result<(), Error> {
            loop {
                tokio::select! {
                    Some(msg) = self.conn.next() => {
                        if let Some(ret) = self.handle_incoming_msg(msg.unwrap()).await {
                            return ret;
                        }
                    }
                    Some(ev) = self.event_rx.recv() => self.publish_event(ev).await,
                    Some(res) = self.terminate_rx.recv() => {
                        self.terminate().await;
                        return res;
                    },
                }
            }
        }

        async fn publish_event(&mut self, ev: Event) {
            let subs_id = match self.subscriptions.get(&ev.query) {
                Some(id) => id.clone(),
                None => return,
            };
            let _ = self.send(Id::Str(subs_id), ev).await;
        }

        async fn handle_incoming_msg(&mut self, msg: Message) -> Option<Result<(), Error>> {
            match msg {
                Message::Text(s) => self.handle_incoming_text_msg(s).await,
                Message::Ping(v) => {
                    let _ = self.conn.send(Message::Pong(v)).await;
                    None
                },
                Message::Close(_) => {
                    self.terminate().await;
                    Some(Ok(()))
                },
                _ => None,
            }
        }

        async fn handle_incoming_text_msg(&mut self, msg: String) -> Option<Result<(), Error>> {
            match serde_json::from_str::<serde_json::Value>(&msg) {
                Ok(json_msg) => {
                    if let Some(json_method) = json_msg.get("method") {
                        match Method::from_str(json_method.as_str().unwrap()) {
                            Ok(method) => match method {
                                Method::Subscribe => {
                                    let req = serde_json::from_str::<
                                        request::Wrapper<subscribe::Request>,
                                    >(&msg)
                                    .unwrap();

                                    self.add_subscription(
                                        req.params().query.clone(),
                                        req.id().to_string(),
                                    );
                                    self.send(req.id().clone(), subscribe::Response {}).await;
                                },
                                Method::Unsubscribe => {
                                    let req = serde_json::from_str::<
                                        request::Wrapper<unsubscribe::Request>,
                                    >(&msg)
                                    .unwrap();

                                    self.remove_subscription(req.params().query.clone());
                                    self.send(req.id().clone(), unsubscribe::Response {}).await;
                                },
                                _ => {
                                    println!("Unsupported method in incoming request: {}", &method);
                                },
                            },
                            Err(e) => {
                                println!(
                                    "Unexpected method in incoming request: {} ({})",
                                    json_method, e
                                );
                            },
                        }
                    }
                },
                Err(e) => {
                    println!("Failed to parse incoming request: {} ({})", &msg, e);
                },
            }
            None
        }

        fn add_subscription(&mut self, query: String, id: String) {
            println!("Adding subscription with ID {} for query: {}", &id, &query);
            self.subscriptions.insert(query, id);
        }

        fn remove_subscription(&mut self, query: String) {
            if let Some(id) = self.subscriptions.remove(&query) {
                println!("Removed subscription {} for query: {}", id, query);
            }
        }

        async fn send<R>(&mut self, id: Id, res: R)
        where
            R: Response,
        {
            self.conn
                .send(Message::Text(
                    serde_json::to_string(&response::Wrapper::new_with_id(id, Some(res), None))
                        .unwrap(),
                ))
                .await
                .unwrap();
        }

        async fn terminate(&mut self) {
            let _ = self
                .conn
                .close(Some(CloseFrame {
                    code: CloseCode::Normal,
                    reason: Default::default(),
                }))
                .await;
        }
    }

    async fn read_json_fixture(name: &str) -> String {
        fs::read_to_string(
            PathBuf::from("./tests/kvstore_fixtures/v0_34").join(name.to_owned() + ".json"),
        )
        .await
        .unwrap()
    }

    async fn read_event(name: &str) -> Event {
        Event::from_string(&read_json_fixture(name).await).unwrap()
    }

    #[tokio::test]
    async fn websocket_client_happy_path() {
        let event1 = read_event("incoming/subscribe_newblock_0").await;
        let event2 = read_event("incoming/subscribe_newblock_1").await;
        let event3 = read_event("incoming/subscribe_newblock_2").await;
        let test_events = vec![event1, event2, event3];

        println!("Starting WebSocket server...");
        let mut server = TestServer::new("127.0.0.1:0").await;
        println!("Creating client RPC WebSocket connection...");
        let (client, driver) = WebSocketClient::new(server.node_addr.clone())
            .await
            .unwrap();
        let driver_handle = tokio::spawn(async move { driver.run().await });

        println!("Initiating subscription for new blocks...");
        let mut subs = client.subscribe(EventType::NewBlock.into()).await.unwrap();

        // Collect all the events from the subscription.
        let subs_collector_hdl = tokio::spawn(async move {
            let mut results = Vec::new();
            while let Some(res) = subs.next().await {
                results.push(res);
                if results.len() == 3 {
                    break;
                }
            }
            results
        });

        println!("Publishing events");
        // Publish the events from this context
        for ev in &test_events {
            server.publish_event(ev.clone()).unwrap();
        }

        println!("Collecting results from subscription...");
        let collected_results = subs_collector_hdl.await.unwrap();

        client.close().unwrap();
        server.terminate().await.unwrap();
        let _ = driver_handle.await.unwrap();
        println!("Closed client and terminated server");

        assert_eq!(3, collected_results.len());
        for i in 0..3 {
            assert_eq!(
                test_events[i],
                collected_results[i].as_ref().unwrap().clone()
            );
        }
    }
}
