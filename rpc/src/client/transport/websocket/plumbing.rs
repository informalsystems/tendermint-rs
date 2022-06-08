use alloc::{borrow::Cow, collections::BTreeMap as HashMap};

use async_tungstenite::{
    tokio::{
        connect_async_with_config, connect_async_with_tls_connector_and_config, ConnectStream,
    },
    tungstenite::protocol::WebSocketConfig,
    tungstenite::{
        protocol::{frame::coding::CloseCode, CloseFrame},
        Message,
    },
    WebSocketStream,
};
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant};
use tracing::{debug, error};

use crate::{
    client::{
        subscription::SubscriptionTx,
        sync::{unbounded, ChannelRx, ChannelTx},
        transport::router::{PublishResult, SubscriptionId, SubscriptionIdRef, SubscriptionRouter},
    },
    endpoint::{subscribe, unsubscribe},
    event::Event,
    prelude::*,
    query::Query,
    request::Wrapper,
    response,
    utils::uuid_str,
    Error, Id, Request, Response, SimpleRequest, Subscription, Url,
};

// WebSocket connection times out if we haven't heard anything at all from the
// server in this long.
//
// Taken from https://github.com/tendermint/tendermint/blob/309e29c245a01825fc9630103311fd04de99fa5e/rpc/jsonrpc/server/ws_handler.go#L27
const RECV_TIMEOUT_SECONDS: u64 = 30;

const RECV_TIMEOUT: Duration = Duration::from_secs(RECV_TIMEOUT_SECONDS);

// How frequently to send ping messages to the WebSocket server.
//
// Taken from https://github.com/tendermint/tendermint/blob/309e29c245a01825fc9630103311fd04de99fa5e/rpc/jsonrpc/server/ws_handler.go#L28
const PING_INTERVAL: Duration = Duration::from_secs((RECV_TIMEOUT_SECONDS * 9) / 10);

/// Marker for the [`AsyncTungsteniteClient`] for clients operating over
/// unsecure connections.
#[derive(Debug, Clone)]
pub struct Unsecure;

/// Marker for the [`AsyncTungsteniteClient`] for clients operating over
/// secure connections.
#[derive(Debug, Clone)]
pub struct Secure;

/// An [`async-tungstenite`]-based WebSocket client.
///
/// Different modes of operation (secure and unsecure) are facilitated by
/// different variants of this type.
///
/// [`async-tungstenite`]: https://crates.io/crates/async-tungstenite
#[derive(Debug, Clone)]
pub struct AsyncTungsteniteClient<C> {
    cmd_tx: ChannelTx<DriverCommand>,
    _client_type: core::marker::PhantomData<C>,
}

impl AsyncTungsteniteClient<Unsecure> {
    /// Construct a WebSocket client. Immediately attempts to open a WebSocket
    /// connection to the node with the given address.
    ///
    /// On success, this returns both a client handle as well as the WebSocket connection driver.
    /// The execution of this driver becomes the responsibility of the client owner, and must be
    /// executed in a separate asynchronous context to the client to ensure it
    /// doesn't block the client.
    pub async fn new(url: Url, config: Option<WebSocketConfig>) -> Result<(Self, Driver), Error> {
        let url = url.to_string();
        debug!("Connecting to unsecure WebSocket endpoint: {}", url);

        let (stream, _response) = connect_async_with_config(url, config)
            .await
            .map_err(Error::tungstenite)?;

        let (cmd_tx, cmd_rx) = unbounded();
        let driver = Driver::new(stream, cmd_rx);
        let client = Self {
            cmd_tx,
            _client_type: Default::default(),
        };

        Ok((client, driver))
    }
}

impl AsyncTungsteniteClient<Secure> {
    /// Construct a WebSocket client. Immediately attempts to open a WebSocket
    /// connection to the node with the given address, but over a secure
    /// connection.
    ///
    /// On success, this returns both a client handle (a `WebSocketClient`
    /// instance) as well as the WebSocket connection driver. The execution of
    /// this driver becomes the responsibility of the client owner, and must be
    /// executed in a separate asynchronous context to the client to ensure it
    /// doesn't block the client.
    pub async fn new(url: Url, config: Option<WebSocketConfig>) -> Result<(Self, Driver), Error> {
        let url = url.to_string();
        debug!("Connecting to secure WebSocket endpoint: {}", url);

        // Not supplying a connector means async_tungstenite will create the
        // connector for us.
        let (stream, _response) = connect_async_with_tls_connector_and_config(url, None, config)
            .await
            .map_err(Error::tungstenite)?;

        let (cmd_tx, cmd_rx) = unbounded();
        let driver = Driver::new(stream, cmd_rx);
        let client = Self {
            cmd_tx,
            _client_type: Default::default(),
        };

        Ok((client, driver))
    }
}

impl<C> AsyncTungsteniteClient<C> {
    fn send_cmd(&self, cmd: DriverCommand) -> Result<(), Error> {
        self.cmd_tx.send(cmd)
    }

    pub async fn perform<R>(&self, request: R) -> Result<R::Response, Error>
    where
        R: SimpleRequest,
    {
        let wrapper = Wrapper::new(request);
        let id = wrapper.id().clone().to_string();
        let wrapped_request = wrapper.into_json();
        let (response_tx, mut response_rx) = unbounded();
        self.send_cmd(DriverCommand::SimpleRequest(SimpleRequestCommand {
            id,
            wrapped_request,
            response_tx,
        }))?;
        let response = response_rx.recv().await.ok_or_else(|| {
            Error::client_internal("failed to hear back from WebSocket driver".to_string())
        })??;
        tracing::debug!("Incoming response: {}", response);
        R::Response::from_string(response)
    }

    pub async fn subscribe(&self, query: Query) -> Result<Subscription, Error> {
        let (subscription_tx, subscription_rx) = unbounded();
        let (response_tx, mut response_rx) = unbounded();
        // By default we use UUIDs to differentiate subscriptions
        let id = uuid_str();
        self.send_cmd(DriverCommand::Subscribe(SubscribeCommand {
            id: id.to_string(),
            query: query.to_string(),
            subscription_tx,
            response_tx,
        }))?;
        // Make sure our subscription request went through successfully.
        let _ = response_rx.recv().await.ok_or_else(|| {
            Error::client_internal("failed to hear back from WebSocket driver".to_string())
        })??;
        Ok(Subscription::new(id, query, subscription_rx))
    }

    pub async fn unsubscribe(&self, query: Query) -> Result<(), Error> {
        let (response_tx, mut response_rx) = unbounded();
        self.send_cmd(DriverCommand::Unsubscribe(UnsubscribeCommand {
            query: query.to_string(),
            response_tx,
        }))?;
        let _ = response_rx.recv().await.ok_or_else(|| {
            Error::client_internal("failed to hear back from WebSocket driver".to_string())
        })??;
        Ok(())
    }

    /// Signals to the driver that it must terminate.
    pub fn close(self) -> Result<(), Error> {
        self.send_cmd(DriverCommand::Terminate)
    }
}

/// Allows us to erase the type signatures associated with the different
/// WebSocket client variants.
#[derive(Debug, Clone)]
pub enum Client {
    Unsecure(AsyncTungsteniteClient<Unsecure>),
    Secure(AsyncTungsteniteClient<Secure>),
}

impl Client {
    pub async fn new_unsecure(
        url: Url,
        config: Option<WebSocketConfig>,
    ) -> Result<(Self, Driver), Error> {
        let (client, driver) = AsyncTungsteniteClient::<Unsecure>::new(url, config).await?;
        Ok((Self::Unsecure(client), driver))
    }

    pub async fn new_secure(
        url: Url,
        config: Option<WebSocketConfig>,
    ) -> Result<(Self, Driver), Error> {
        let (client, driver) = AsyncTungsteniteClient::<Secure>::new(url, config).await?;
        Ok((Self::Secure(client), driver))
    }

    pub async fn perform<R>(&self, request: R) -> Result<R::Response, Error>
    where
        R: SimpleRequest,
    {
        match self {
            Client::Unsecure(c) => c.perform(request).await,
            Client::Secure(c) => c.perform(request).await,
        }
    }

    pub async fn subscribe(&self, query: Query) -> Result<Subscription, Error> {
        match self {
            Client::Unsecure(c) => c.subscribe(query).await,
            Client::Secure(c) => c.subscribe(query).await,
        }
    }

    pub async fn unsubscribe(&self, query: Query) -> Result<(), Error> {
        match self {
            Client::Unsecure(c) => c.unsubscribe(query).await,
            Client::Secure(c) => c.unsubscribe(query).await,
        }
    }

    pub fn close(self) -> Result<(), Error> {
        match self {
            Client::Unsecure(c) => c.close(),
            Client::Secure(c) => c.close(),
        }
    }
}

// The different types of commands that can be sent from the WebSocketClient to
// the driver.
#[derive(Debug, Clone)]
enum DriverCommand {
    // Initiate a subscription request.
    Subscribe(SubscribeCommand),
    // Initiate an unsubscribe request.
    Unsubscribe(UnsubscribeCommand),
    // For non-subscription-related requests.
    SimpleRequest(SimpleRequestCommand),
    Terminate,
}

#[derive(Debug, Clone)]
struct SubscribeCommand {
    // The desired ID for the outgoing JSON-RPC request.
    id: String,
    // The query for which we want to receive events.
    query: String,
    // Where to send subscription events.
    subscription_tx: SubscriptionTx,
    // Where to send the result of the subscription request.
    response_tx: ChannelTx<Result<(), Error>>,
}

#[derive(Debug, Clone)]
struct UnsubscribeCommand {
    // The query from which to unsubscribe.
    query: String,
    // Where to send the result of the unsubscribe request.
    response_tx: ChannelTx<Result<(), Error>>,
}

#[derive(Debug, Clone)]
struct SimpleRequestCommand {
    // The desired ID for the outgoing JSON-RPC request. Technically we
    // could extract this from the wrapped request, but that would mean
    // additional unnecessary computational resources for deserialization.
    id: String,
    // The wrapped and serialized JSON-RPC request.
    wrapped_request: String,
    // Where to send the result of the simple request.
    response_tx: ChannelTx<Result<String, Error>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GenericJsonResponse(serde_json::Value);

impl Response for GenericJsonResponse {}

/// Drives the WebSocket connection for a `Client` instance.
///
/// This is the primary component responsible for transport-level interaction
/// with the remote WebSocket endpoint.
pub struct Driver {
    // The underlying WebSocket network connection.
    stream: WebSocketStream<ConnectStream>,
    // Facilitates routing of events to their respective subscriptions.
    router: SubscriptionRouter,
    // How we receive incoming commands from the Client.
    cmd_rx: ChannelRx<DriverCommand>,
    // Commands we've received but have not yet completed, indexed by their ID.
    // A Terminate command is executed immediately.
    pending_commands: HashMap<SubscriptionId, DriverCommand>,
}

impl Driver {
    fn new(stream: WebSocketStream<ConnectStream>, cmd_rx: ChannelRx<DriverCommand>) -> Self {
        Self {
            stream,
            router: SubscriptionRouter::default(),
            cmd_rx,
            pending_commands: HashMap::new(),
        }
    }

    /// Executes the WebSocket driver, which manages the underlying WebSocket
    /// transport.
    pub async fn run(mut self) -> Result<(), Error> {
        let mut ping_interval =
            tokio::time::interval_at(Instant::now() + PING_INTERVAL, PING_INTERVAL);

        let recv_timeout = tokio::time::sleep(RECV_TIMEOUT);
        tokio::pin!(recv_timeout);

        loop {
            tokio::select! {
                Some(res) = self.stream.next() => match res {
                    Ok(msg) => {
                        // Reset the receive timeout every time we successfully
                        // receive a message from the remote endpoint.
                        recv_timeout.as_mut().reset(Instant::now() + RECV_TIMEOUT);
                        self.handle_incoming_msg(msg).await?
                    },
                    Err(e) => return Err(
                        Error::web_socket(
                            "failed to read from WebSocket connection".to_string(),
                            e
                        ),
                    ),
                },
                Some(cmd) = self.cmd_rx.recv() => match cmd {
                    DriverCommand::Subscribe(subs_cmd) => self.subscribe(subs_cmd).await?,
                    DriverCommand::Unsubscribe(unsubs_cmd) => self.unsubscribe(unsubs_cmd).await?,
                    DriverCommand::SimpleRequest(req_cmd) => self.simple_request(req_cmd).await?,
                    DriverCommand::Terminate => return self.close().await,
                },
                _ = ping_interval.tick() => self.ping().await?,
                _ = &mut recv_timeout => {
                    return Err(Error::web_socket_timeout(RECV_TIMEOUT));
                }
            }
        }
    }

    async fn send_msg(&mut self, msg: Message) -> Result<(), Error> {
        self.stream.send(msg).await.map_err(|e| {
            Error::web_socket("failed to write to WebSocket connection".to_string(), e)
        })
    }

    async fn send_request<R>(&mut self, wrapper: Wrapper<R>) -> Result<(), Error>
    where
        R: Request,
    {
        self.send_msg(Message::Text(
            serde_json::to_string_pretty(&wrapper).unwrap(),
        ))
        .await
    }

    async fn subscribe(&mut self, cmd: SubscribeCommand) -> Result<(), Error> {
        // If we already have an active subscription for the given query,
        // there's no need to initiate another one. Just add this subscription
        // to the router.
        if self.router.num_subscriptions_for_query(cmd.query.clone()) > 0 {
            let (id, query, subscription_tx, response_tx) =
                (cmd.id, cmd.query, cmd.subscription_tx, cmd.response_tx);
            self.router.add(id, query, subscription_tx);
            return response_tx.send(Ok(()));
        }

        // Otherwise, we need to initiate a subscription request.
        let wrapper = Wrapper::new_with_id(
            Id::Str(cmd.id.clone()),
            subscribe::Request::new(cmd.query.clone()),
        );
        if let Err(e) = self.send_request(wrapper).await {
            cmd.response_tx.send(Err(e.clone()))?;
            return Err(e);
        }
        self.pending_commands
            .insert(cmd.id.clone(), DriverCommand::Subscribe(cmd));
        Ok(())
    }

    async fn unsubscribe(&mut self, cmd: UnsubscribeCommand) -> Result<(), Error> {
        // Terminate all subscriptions for this query immediately. This
        // prioritizes acknowledgement of the caller's wishes over networking
        // problems.
        if self.router.remove_by_query(cmd.query.clone()) == 0 {
            // If there were no subscriptions for this query, respond
            // immediately.
            cmd.response_tx.send(Ok(()))?;
            return Ok(());
        }

        // Unsubscribe requests can (and probably should) have distinct
        // JSON-RPC IDs as compared to their subscription IDs.
        let wrapper = Wrapper::new(unsubscribe::Request::new(cmd.query.clone()));
        let req_id = wrapper.id().clone();
        if let Err(e) = self.send_request(wrapper).await {
            cmd.response_tx.send(Err(e.clone()))?;
            return Err(e);
        }
        self.pending_commands
            .insert(req_id.to_string(), DriverCommand::Unsubscribe(cmd));
        Ok(())
    }

    async fn simple_request(&mut self, cmd: SimpleRequestCommand) -> Result<(), Error> {
        if let Err(e) = self
            .send_msg(Message::Text(cmd.wrapped_request.clone()))
            .await
        {
            cmd.response_tx.send(Err(e.clone()))?;
            return Err(e);
        }
        self.pending_commands
            .insert(cmd.id.clone(), DriverCommand::SimpleRequest(cmd));
        Ok(())
    }

    async fn handle_incoming_msg(&mut self, msg: Message) -> Result<(), Error> {
        match msg {
            Message::Text(s) => self.handle_text_msg(s).await,
            Message::Ping(v) => self.pong(v).await,
            _ => Ok(()),
        }
    }

    async fn handle_text_msg(&mut self, msg: String) -> Result<(), Error> {
        if let Ok(ev) = Event::from_string(&msg) {
            self.publish_event(ev).await;
            return Ok(());
        }

        let wrapper: response::Wrapper<GenericJsonResponse> = match serde_json::from_str(&msg) {
            Ok(w) => w,
            Err(e) => {
                error!(
                    "Failed to deserialize incoming message as a JSON-RPC message: {}",
                    e
                );

                debug!("JSON-RPC message: {}", msg);

                return Ok(());
            },
        };

        debug!("Generic JSON-RPC message: {:?}", wrapper);

        let id = wrapper.id().to_string();

        if let Some(e) = wrapper.into_error() {
            self.publish_error(&id, e).await;
        }

        if let Some(pending_cmd) = self.pending_commands.remove(&id) {
            self.respond_to_pending_command(pending_cmd, msg).await?;
        };

        // We ignore incoming messages whose ID we don't recognize (could be
        // relating to a fire-and-forget unsubscribe request - see the
        // publish_event() method below).
        Ok(())
    }

    async fn publish_error(&mut self, id: SubscriptionIdRef<'_>, err: Error) {
        if let PublishResult::AllDisconnected(query) = self.router.publish_error(id, err) {
            debug!(
                "All subscribers for query \"{}\" have disconnected. Unsubscribing from query...",
                query
            );

            // If all subscribers have disconnected for this query, we need to
            // unsubscribe from it. We issue a fire-and-forget unsubscribe
            // message.
            if let Err(e) = self
                .send_request(Wrapper::new(unsubscribe::Request::new(query)))
                .await
            {
                error!("Failed to send unsubscribe request: {}", e);
            }
        }
    }

    async fn publish_event(&mut self, ev: Event) {
        if let PublishResult::AllDisconnected(query) = self.router.publish_event(ev) {
            debug!(
                "All subscribers for query \"{}\" have disconnected. Unsubscribing from query...",
                query
            );

            // If all subscribers have disconnected for this query, we need to
            // unsubscribe from it. We issue a fire-and-forget unsubscribe
            // message.
            if let Err(e) = self
                .send_request(Wrapper::new(unsubscribe::Request::new(query)))
                .await
            {
                error!("Failed to send unsubscribe request: {}", e);
            }
        }
    }

    async fn respond_to_pending_command(
        &mut self,
        pending_cmd: DriverCommand,
        response: String,
    ) -> Result<(), Error> {
        match pending_cmd {
            DriverCommand::Subscribe(cmd) => {
                let (id, query, subscription_tx, response_tx) =
                    (cmd.id, cmd.query, cmd.subscription_tx, cmd.response_tx);
                self.router.add(id, query, subscription_tx);
                response_tx.send(Ok(()))
            },
            DriverCommand::Unsubscribe(cmd) => cmd.response_tx.send(Ok(())),
            DriverCommand::SimpleRequest(cmd) => cmd.response_tx.send(Ok(response)),
            _ => Ok(()),
        }
    }

    async fn pong(&mut self, v: Vec<u8>) -> Result<(), Error> {
        self.send_msg(Message::Pong(v)).await
    }

    async fn ping(&mut self) -> Result<(), Error> {
        self.send_msg(Message::Ping(Vec::new())).await
    }

    async fn close(mut self) -> Result<(), Error> {
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
