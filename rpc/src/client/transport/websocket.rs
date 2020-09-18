//! WebSocket-based clients for accessing Tendermint RPC functionality.

use crate::client::subscription::{
    SubscriptionDriverCmd, SubscriptionState, TwoPhaseSubscriptionRouter,
};
use crate::client::sync::{unbounded, ChannelRx, ChannelTx};
use crate::client::transport::get_tcp_host_port;
use crate::endpoint::{subscribe, unsubscribe};
use crate::event::Event;
use crate::{
    request, response, Error, Response, Result, Subscription, SubscriptionClient, SubscriptionId,
};
use async_trait::async_trait;
use async_tungstenite::tokio::{connect_async, TokioAdapter};
use async_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use async_tungstenite::tungstenite::protocol::CloseFrame;
use async_tungstenite::tungstenite::Message;
use async_tungstenite::WebSocketStream;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::str::FromStr;
use tendermint::net;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;

/// Tendermint RPC client that provides [`Event`] subscription capabilities
/// over JSON-RPC over a WebSocket connection.
///
/// In order to not block the calling task, this client spawns an asynchronous
/// driver that continuously interacts with the actual WebSocket connection.
/// The `WebSocketClient` itself is effectively just a handle to this driver.
/// This driver is spawned as the client is created.
///
/// To terminate the client and the driver, simply use its [`close`] method.
///
/// ## Examples
///
/// ```rust,ignore
/// use tendermint_rpc::{WebSocketClient, SubscriptionClient};
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() {
///     let mut client = WebSocketClient::new("tcp://127.0.0.1:26657".parse().unwrap())
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
/// [`close`]: struct.WebSocketClient.html#method.close
#[derive(Debug)]
pub struct WebSocketClient {
    host: String,
    port: u16,
    driver_handle: JoinHandle<Result<()>>,
    cmd_tx: ChannelTx<SubscriptionDriverCmd>,
}

impl WebSocketClient {
    /// Construct a WebSocket client. Immediately attempts to open a WebSocket
    /// connection to the node with the given address.
    pub async fn new(address: net::Address) -> Result<Self> {
        let (host, port) = get_tcp_host_port(address)?;
        let (stream, _response) =
            connect_async(&format!("ws://{}:{}/websocket", &host, port)).await?;
        let (cmd_tx, cmd_rx) = unbounded();
        let driver = WebSocketSubscriptionDriver::new(stream, cmd_rx);
        let driver_handle = tokio::spawn(async move { driver.run().await });
        Ok(Self {
            host,
            port,
            driver_handle,
            cmd_tx,
        })
    }

    async fn send_cmd(&mut self, cmd: SubscriptionDriverCmd) -> Result<()> {
        self.cmd_tx.send(cmd).await.map_err(|e| {
            Error::client_internal_error(format!("failed to send command to client driver: {}", e))
        })
    }

    /// Attempt to gracefully close the WebSocket connection.
    pub async fn close(mut self) -> Result<()> {
        self.send_cmd(SubscriptionDriverCmd::Terminate).await?;
        self.driver_handle.await.map_err(|e| {
            Error::client_internal_error(format!(
                "failed while waiting for WebSocket driver task to terminate: {}",
                e
            ))
        })?
    }
}

#[async_trait]
impl SubscriptionClient for WebSocketClient {
    async fn subscribe(&mut self, query: String) -> Result<Subscription> {
        let (event_tx, event_rx) = unbounded();
        let (result_tx, mut result_rx) = unbounded::<Result<()>>();
        let id = SubscriptionId::default();
        self.send_cmd(SubscriptionDriverCmd::Subscribe {
            id: id.clone(),
            query: query.clone(),
            event_tx,
            result_tx,
        })
        .await?;
        // Wait to make sure our subscription request went through
        // successfully.
        result_rx.recv().await.ok_or_else(|| {
            Error::client_internal_error("failed to hear back from WebSocket driver".to_string())
        })??;
        Ok(Subscription::new(id, query, event_rx, self.cmd_tx.clone()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GenericJSONResponse(serde_json::Value);

impl Response for GenericJSONResponse {}

#[derive(Debug)]
struct WebSocketSubscriptionDriver {
    stream: WebSocketStream<TokioAdapter<TcpStream>>,
    router: TwoPhaseSubscriptionRouter,
    cmd_rx: ChannelRx<SubscriptionDriverCmd>,
}

impl WebSocketSubscriptionDriver {
    fn new(
        stream: WebSocketStream<TokioAdapter<TcpStream>>,
        cmd_rx: ChannelRx<SubscriptionDriverCmd>,
    ) -> Self {
        Self {
            stream,
            router: TwoPhaseSubscriptionRouter::default(),
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
                Some(cmd) = self.cmd_rx.recv() => match cmd {
                    SubscriptionDriverCmd::Subscribe {
                        id,
                        query,
                        event_tx,
                        result_tx,
                    } => self.subscribe(id, query, event_tx, result_tx).await?,
                    SubscriptionDriverCmd::Unsubscribe {
                        id,
                        query,
                        result_tx,
                    } => self.unsubscribe(id, query, result_tx).await?,
                    SubscriptionDriverCmd::Terminate => return self.close().await,
                },
            }
        }
    }

    async fn send_msg(&mut self, msg: Message) -> Result<()> {
        self.stream.send(msg).await.map_err(|e| {
            Error::websocket_error(format!("failed to write to WebSocket connection: {}", e))
        })
    }

    async fn send_request<R>(
        &mut self,
        req: request::Wrapper<R>,
        result_tx: &mut ChannelTx<Result<()>>,
    ) -> Result<()>
    where
        R: request::Request,
    {
        if let Err(e) = self
            .send_msg(Message::Text(serde_json::to_string_pretty(&req).unwrap()))
            .await
        {
            let _ = result_tx.send(Err(e.clone())).await;
            return Err(e);
        }
        Ok(())
    }

    async fn subscribe(
        &mut self,
        id: SubscriptionId,
        query: String,
        event_tx: ChannelTx<Result<Event>>,
        mut result_tx: ChannelTx<Result<()>>,
    ) -> Result<()> {
        let req = request::Wrapper::new_with_id(
            id.clone().into(),
            subscribe::Request::new(query.clone()),
        );
        let _ = self.send_request(req, &mut result_tx).await;
        self.router
            .pending_add(id.as_str(), &id, query, event_tx, result_tx);
        Ok(())
    }

    async fn unsubscribe(
        &mut self,
        id: SubscriptionId,
        query: String,
        mut result_tx: ChannelTx<Result<()>>,
    ) -> Result<()> {
        let req = request::Wrapper::new(unsubscribe::Request::new(query.clone()));
        let req_id = req.id().to_string();
        let _ = self.send_request(req, &mut result_tx).await;
        self.router.pending_remove(&req_id, &id, query, result_tx);
        Ok(())
    }

    async fn handle_incoming_msg(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Text(s) => self.handle_text_msg(s).await,
            Message::Ping(v) => self.pong(v).await?,
            _ => (),
        }
        Ok(())
    }

    async fn handle_text_msg(&mut self, msg: String) {
        // See https://github.com/tendermint/tendermint/issues/5373
        let msgs = msg
            // TODO(thane): Find a better way of dealing with this. This is
            //              brittle and will break if there's even just one
            //              string in the message that contains the same pattern.
            .split("}\n{")
            .map(|m| m.to_string())
            .collect::<Vec<String>>();
        for mut msg in msgs {
            if !msg.starts_with('{') {
                msg.insert(0, '{');
            }
            if !msg.ends_with('}') {
                msg.push('}');
            }
            match Event::from_string(&msg) {
                Ok(ev) => {
                    self.router.publish(ev).await;
                }
                Err(_) => {
                    if let Ok(wrapper) =
                        serde_json::from_str::<response::Wrapper<GenericJSONResponse>>(&msg)
                    {
                        self.handle_generic_response(wrapper).await;
                    }
                }
            }
        }
    }

    async fn handle_generic_response(&mut self, wrapper: response::Wrapper<GenericJSONResponse>) {
        let req_id = wrapper.id().to_string();
        if let Some(state) = self.router.subscription_state(&req_id) {
            match wrapper.into_result() {
                Ok(_) => match state {
                    SubscriptionState::Pending => {
                        let _ = self.router.confirm_add(&req_id).await;
                    }
                    SubscriptionState::Cancelling => {
                        let _ = self.router.confirm_remove(&req_id).await;
                    }
                    SubscriptionState::Active => {
                        if let Some(event_tx) = self.router.get_active_subscription_mut(
                            &SubscriptionId::from_str(&req_id).unwrap(),
                        ) {
                            let _ = event_tx.send(
                                Err(Error::websocket_error(
                                    "failed to parse incoming response from remote WebSocket endpoint - does this client support the remote's RPC version?",
                                )),
                            ).await;
                        }
                    }
                },
                Err(e) => match state {
                    SubscriptionState::Pending => {
                        let _ = self.router.cancel_add(&req_id, e).await;
                    }
                    SubscriptionState::Cancelling => {
                        let _ = self.router.cancel_remove(&req_id, e).await;
                    }
                    // This is important to allow the remote endpoint to
                    // arbitrarily send error responses back to specific
                    // subscriptions.
                    SubscriptionState::Active => {
                        if let Some(event_tx) = self.router.get_active_subscription_mut(
                            &SubscriptionId::from_str(&req_id).unwrap(),
                        ) {
                            // TODO(thane): Does an error here warrant terminating the subscription, or the driver?
                            let _ = event_tx.send(Err(e)).await;
                        }
                    }
                },
            }
        }
    }

    async fn pong(&mut self, v: Vec<u8>) -> Result<()> {
        self.send_msg(Message::Pong(v)).await
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Id, Method};
    use async_tungstenite::tokio::accept_async;
    use futures::StreamExt;
    use std::collections::HashMap;
    use std::convert::TryInto;
    use std::path::PathBuf;
    use std::str::FromStr;
    use tokio::fs;
    use tokio::net::TcpListener;

    // Interface to a driver that manages all incoming WebSocket connections.
    struct TestServer {
        node_addr: net::Address,
        driver_hdl: JoinHandle<Result<()>>,
        terminate_tx: ChannelTx<Result<()>>,
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

        async fn publish_event(&mut self, ev: Event) -> Result<()> {
            self.event_tx.send(ev).await
        }

        async fn terminate(mut self) -> Result<()> {
            self.terminate_tx.send(Ok(())).await.unwrap();
            self.driver_hdl.await.unwrap()
        }
    }

    // Manages all incoming WebSocket connections.
    struct TestServerDriver {
        listener: TcpListener,
        event_rx: ChannelRx<Event>,
        terminate_rx: ChannelRx<Result<()>>,
        handlers: Vec<TestServerHandler>,
    }

    impl TestServerDriver {
        fn new(
            listener: TcpListener,
            event_rx: ChannelRx<Event>,
            terminate_rx: ChannelRx<Result<()>>,
        ) -> Self {
            Self {
                listener,
                event_rx,
                terminate_rx,
                handlers: Vec::new(),
            }
        }

        async fn run(mut self) -> Result<()> {
            loop {
                tokio::select! {
                    Some(ev) = self.event_rx.recv() => self.publish_event(ev).await,
                    Some(res) = self.listener.next() => self.handle_incoming(res.unwrap()).await,
                    Some(res) = self.terminate_rx.recv() => {
                        self.terminate().await;
                        return res;
                    },
                }
            }
        }

        // Publishes the given event to all subscribers for the query relating
        // to the event.
        async fn publish_event(&mut self, ev: Event) {
            for handler in &mut self.handlers {
                handler.publish_event(ev.clone()).await;
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
        driver_hdl: JoinHandle<Result<()>>,
        terminate_tx: ChannelTx<Result<()>>,
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

        async fn publish_event(&mut self, ev: Event) {
            let _ = self.event_tx.send(ev).await;
        }

        async fn terminate(mut self) -> Result<()> {
            self.terminate_tx.send(Ok(())).await?;
            self.driver_hdl.await.unwrap()
        }
    }

    // Manages interaction with a single incoming WebSocket connection.
    struct TestServerHandlerDriver {
        conn: WebSocketStream<TokioAdapter<TcpStream>>,
        event_rx: ChannelRx<Event>,
        terminate_rx: ChannelRx<Result<()>>,
        // A mapping of subscription queries to subscription IDs for this
        // connection.
        subscriptions: HashMap<String, SubscriptionId>,
    }

    impl TestServerHandlerDriver {
        fn new(
            conn: WebSocketStream<TokioAdapter<TcpStream>>,
            event_rx: ChannelRx<Event>,
            terminate_rx: ChannelRx<Result<()>>,
        ) -> Self {
            Self {
                conn,
                event_rx,
                terminate_rx,
                subscriptions: HashMap::new(),
            }
        }

        async fn run(mut self) -> Result<()> {
            loop {
                tokio::select! {
                    Some(res) = self.conn.next() => {
                        if let Some(ret) = self.handle_incoming_msg(res.unwrap()).await {
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
            let _ = self.send(subs_id.into(), ev).await;
        }

        async fn handle_incoming_msg(&mut self, msg: Message) -> Option<Result<()>> {
            match msg {
                Message::Text(s) => self.handle_incoming_text_msg(s).await,
                Message::Ping(v) => {
                    let _ = self.conn.send(Message::Pong(v)).await;
                    None
                }
                Message::Close(_) => {
                    self.terminate().await;
                    Some(Ok(()))
                }
                _ => None,
            }
        }

        async fn handle_incoming_text_msg(&mut self, msg: String) -> Option<Result<()>> {
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
                                        req.id().clone().try_into().unwrap(),
                                    );
                                    self.send(req.id().clone(), subscribe::Response {}).await;
                                }
                                Method::Unsubscribe => {
                                    let req = serde_json::from_str::<
                                        request::Wrapper<unsubscribe::Request>,
                                    >(&msg)
                                    .unwrap();

                                    self.remove_subscription(req.params().query.clone());
                                    self.send(req.id().clone(), unsubscribe::Response {}).await;
                                }
                                _ => {
                                    println!("Unsupported method in incoming request: {}", &method);
                                }
                            },
                            Err(e) => {
                                println!(
                                    "Unexpected method in incoming request: {} ({})",
                                    json_method, e
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to parse incoming request: {} ({})", &msg, e);
                }
            }
            None
        }

        fn add_subscription(&mut self, query: String, id: SubscriptionId) {
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
        fs::read_to_string(PathBuf::from("./tests/support/").join(name.to_owned() + ".json"))
            .await
            .unwrap()
    }

    async fn read_event(name: &str) -> Event {
        Event::from_string(&read_json_fixture(name).await).unwrap()
    }

    #[tokio::test]
    async fn websocket_client_happy_path() {
        let event1 = read_event("event_new_block_1").await;
        let event2 = read_event("event_new_block_2").await;
        let event3 = read_event("event_new_block_3").await;
        let test_events = vec![event1, event2, event3];

        println!("Starting WebSocket server...");
        let mut server = TestServer::new("127.0.0.1:0").await;
        println!("Creating client RPC WebSocket connection...");
        let mut client = WebSocketClient::new(server.node_addr.clone())
            .await
            .unwrap();

        println!("Initiating subscription for new blocks...");
        let mut subs = client
            .subscribe("tm.event='NewBlock'".to_string())
            .await
            .unwrap();

        // Collect all the events from the subscription.
        let subs_collector_hdl = tokio::spawn(async move {
            let mut results = Vec::new();
            while let Some(res) = subs.next().await {
                results.push(res);
                if results.len() == 3 {
                    break;
                }
            }
            println!("Terminating subscription...");
            subs.terminate().await.unwrap();
            results
        });

        println!("Publishing events");
        // Publish the events from this context
        for ev in &test_events {
            server.publish_event(ev.clone()).await.unwrap();
        }

        println!("Collecting results from subscription...");
        let collected_results = subs_collector_hdl.await.unwrap();

        client.close().await.unwrap();
        server.terminate().await.unwrap();
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
