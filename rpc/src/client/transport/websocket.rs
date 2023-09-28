//! WebSocket-based clients for accessing Tendermint RPC functionality.

use alloc::{borrow::Cow, collections::BTreeMap as HashMap, fmt};
use core::{
    convert::{TryFrom, TryInto},
    ops::Add,
    str::FromStr,
};

use async_trait::async_trait;
use async_tungstenite::{
    tokio::ConnectStream,
    tungstenite::{
        protocol::{frame::coding::CloseCode, CloseFrame},
        Message,
    },
    WebSocketStream,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant};
use tracing::{debug, error};

use tendermint::{block::Height, Hash};
use tendermint_config::net;

use super::router::{SubscriptionId, SubscriptionIdRef};
use crate::{
    client::{
        subscription::SubscriptionTx,
        sync::{ChannelRx, ChannelTx},
        transport::router::{PublishResult, SubscriptionRouter},
        Client, CompatMode,
    },
    dialect::{v0_34, Dialect, LatestDialect},
    endpoint::{self, subscribe, unsubscribe},
    error::Error,
    event::{self, Event},
    prelude::*,
    query::Query,
    request::Wrapper,
    response, Id, Order, Request, Response, Scheme, SimpleRequest, Subscription,
    SubscriptionClient, Url,
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

/// Low-level WebSocket configuration
pub use async_tungstenite::tungstenite::protocol::WebSocketConfig;

/// Tendermint RPC client that provides access to all RPC functionality
/// (including [`Event`] subscription) over a WebSocket connection.
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
    inner: sealed::WebSocketClient,
    compat: CompatMode,
}

/// The builder pattern constructor for [`WebSocketClient`].
pub struct Builder {
    url: WebSocketClientUrl,
    compat: CompatMode,
    transport_config: Option<WebSocketConfig>,
}

impl Builder {
    /// Use the specified compatibility mode for the Tendermint RPC protocol.
    ///
    /// The default is the latest protocol version supported by this crate.
    pub fn compat_mode(mut self, mode: CompatMode) -> Self {
        self.compat = mode;
        self
    }

    /// Use the specfied low-level WebSocket configuration options.
    pub fn config(mut self, config: WebSocketConfig) -> Self {
        self.transport_config = Some(config);
        self
    }

    /// Try to create a client with the options specified for this builder.
    pub async fn build(self) -> Result<(WebSocketClient, WebSocketClientDriver), Error> {
        let url = self.url.0;
        let compat = self.compat;
        let (inner, driver) = if url.is_secure() {
            sealed::WebSocketClient::new_secure(url, compat, self.transport_config).await?
        } else {
            sealed::WebSocketClient::new_unsecure(url, compat, self.transport_config).await?
        };

        Ok((WebSocketClient { inner, compat }, driver))
    }
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
        let url = url.try_into()?;
        Self::builder(url).build().await
    }

    /// Construct a new WebSocket-based client connecting to the given
    /// Tendermint node's RPC endpoint.
    ///
    /// Supports both `ws://` and `wss://` protocols.
    pub async fn new_with_config<U>(
        url: U,
        config: WebSocketConfig,
    ) -> Result<(Self, WebSocketClientDriver), Error>
    where
        U: TryInto<WebSocketClientUrl, Error = Error>,
    {
        let url = url.try_into()?;
        Self::builder(url).config(config).build().await
    }

    /// Initiate a builder for a WebSocket-based client connecting to the given
    /// Tendermint node's RPC endpoint.
    ///
    /// Supports both `ws://` and `wss://` protocols.
    pub fn builder(url: WebSocketClientUrl) -> Builder {
        Builder {
            url,
            compat: Default::default(),
            transport_config: Default::default(),
        }
    }

    async fn perform_with_dialect<R, S>(&self, request: R, dialect: S) -> Result<R::Output, Error>
    where
        R: SimpleRequest<S>,
        S: Dialect,
    {
        self.inner.perform(request, dialect).await
    }
}

#[async_trait]
impl Client for WebSocketClient {
    async fn perform<R>(&self, request: R) -> Result<R::Output, Error>
    where
        R: SimpleRequest,
    {
        self.perform_with_dialect(request, LatestDialect).await
    }

    async fn block_results<H>(&self, height: H) -> Result<endpoint::block_results::Response, Error>
    where
        H: Into<Height> + Send,
    {
        perform_with_compat!(self, endpoint::block_results::Request::new(height.into()))
    }

    async fn latest_block_results(&self) -> Result<endpoint::block_results::Response, Error> {
        perform_with_compat!(self, endpoint::block_results::Request::default())
    }

    async fn header<H>(&self, height: H) -> Result<endpoint::header::Response, Error>
    where
        H: Into<Height> + Send,
    {
        let height = height.into();
        match self.compat {
            CompatMode::V0_37 => self.perform(endpoint::header::Request::new(height)).await,
            CompatMode::V0_34 => {
                // Back-fill with a request to /block endpoint and
                // taking just the header from the response.
                let resp = self
                    .perform_with_dialect(endpoint::block::Request::new(height), v0_34::Dialect)
                    .await?;
                Ok(resp.into())
            },
        }
    }

    async fn header_by_hash(
        &self,
        hash: Hash,
    ) -> Result<endpoint::header_by_hash::Response, Error> {
        match self.compat {
            CompatMode::V0_37 => {
                self.perform(endpoint::header_by_hash::Request::new(hash))
                    .await
            },
            CompatMode::V0_34 => {
                // Back-fill with a request to /block_by_hash endpoint and
                // taking just the header from the response.
                let resp = self
                    .perform_with_dialect(
                        endpoint::block_by_hash::Request::new(hash),
                        v0_34::Dialect,
                    )
                    .await?;
                Ok(resp.into())
            },
        }
    }

    async fn tx(&self, hash: Hash, prove: bool) -> Result<endpoint::tx::Response, Error> {
        perform_with_compat!(self, endpoint::tx::Request::new(hash, prove))
    }

    async fn tx_search(
        &self,
        query: Query,
        prove: bool,
        page: u32,
        per_page: u8,
        order: Order,
    ) -> Result<endpoint::tx_search::Response, Error> {
        perform_with_compat!(
            self,
            endpoint::tx_search::Request::new(query, prove, page, per_page, order)
        )
    }

    async fn broadcast_tx_commit<T>(
        &self,
        tx: T,
    ) -> Result<endpoint::broadcast::tx_commit::Response, Error>
    where
        T: Into<Vec<u8>> + Send,
    {
        perform_with_compat!(self, endpoint::broadcast::tx_commit::Request::new(tx))
    }
}

#[async_trait]
impl SubscriptionClient for WebSocketClient {
    async fn subscribe(&self, query: Query) -> Result<Subscription, Error> {
        self.inner.subscribe(query).await
    }

    async fn unsubscribe(&self, query: Query) -> Result<(), Error> {
        self.inner.unsubscribe(query).await
    }

    fn close(self) -> Result<(), Error> {
        self.inner.close()
    }
}

/// A URL limited to use with WebSocket clients.
///
/// Facilitates useful type conversions and inferences.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WebSocketClientUrl(Url);

impl TryFrom<Url> for WebSocketClientUrl {
    type Error = Error;

    fn try_from(value: Url) -> Result<Self, Error> {
        match value.scheme() {
            Scheme::WebSocket | Scheme::SecureWebSocket => Ok(Self(value)),
            _ => Err(Error::invalid_params(format!(
                "cannot use URL {value} with WebSocket clients"
            ))),
        }
    }
}

impl FromStr for WebSocketClientUrl {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let url: Url = s.parse()?;
        url.try_into()
    }
}

impl fmt::Display for WebSocketClientUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<&str> for WebSocketClientUrl {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Error> {
        value.parse()
    }
}

impl TryFrom<net::Address> for WebSocketClientUrl {
    type Error = Error;

    fn try_from(value: net::Address) -> Result<Self, Error> {
        match value {
            net::Address::Tcp {
                peer_id: _,
                host,
                port,
            } => format!("ws://{host}:{port}/websocket").parse(),
            net::Address::Unix { .. } => Err(Error::invalid_params(
                "only TCP-based node addresses are supported".to_string(),
            )),
        }
    }
}

impl From<WebSocketClientUrl> for Url {
    fn from(url: WebSocketClientUrl) -> Self {
        url.0
    }
}

mod sealed {
    use async_tungstenite::{
        tokio::{connect_async_with_config, connect_async_with_tls_connector_and_config},
        tungstenite::client::IntoClientRequest,
    };
    use tracing::debug;

    use super::{
        DriverCommand, SimpleRequestCommand, SubscribeCommand, UnsubscribeCommand,
        WebSocketClientDriver, WebSocketConfig,
    };
    use crate::{
        client::{
            sync::{unbounded, ChannelTx},
            transport::auth::authorize,
            CompatMode,
        },
        dialect::Dialect,
        prelude::*,
        query::Query,
        request::Wrapper,
        utils::uuid_str,
        Error, Response, SimpleRequest, Subscription, Url,
    };

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
        /// On success, this returns both a client handle (a `WebSocketClient`
        /// instance) as well as the WebSocket connection driver. The execution of
        /// this driver becomes the responsibility of the client owner, and must be
        /// executed in a separate asynchronous context to the client to ensure it
        /// doesn't block the client.
        pub async fn new(
            url: Url,
            compat: CompatMode,
            config: Option<WebSocketConfig>,
        ) -> Result<(Self, WebSocketClientDriver), Error> {
            debug!("Connecting to unsecure WebSocket endpoint: {}", url);

            let (stream, _response) = connect_async_with_config(url, config)
                .await
                .map_err(Error::tungstenite)?;

            let (cmd_tx, cmd_rx) = unbounded();
            let driver = WebSocketClientDriver::new(stream, cmd_rx, compat);
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
        pub async fn new(
            url: Url,
            compat: CompatMode,
            config: Option<WebSocketConfig>,
        ) -> Result<(Self, WebSocketClientDriver), Error> {
            debug!("Connecting to secure WebSocket endpoint: {}", url);

            // Not supplying a connector means async_tungstenite will create the
            // connector for us.
            let (stream, _response) =
                connect_async_with_tls_connector_and_config(url, None, config)
                    .await
                    .map_err(Error::tungstenite)?;

            let (cmd_tx, cmd_rx) = unbounded();
            let driver = WebSocketClientDriver::new(stream, cmd_rx, compat);
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

        /// Signals to the driver that it must terminate.
        pub fn close(self) -> Result<(), Error> {
            self.send_cmd(DriverCommand::Terminate)
        }
    }

    impl<C> AsyncTungsteniteClient<C> {
        pub async fn perform<R, S>(&self, request: R) -> Result<R::Output, Error>
        where
            R: SimpleRequest<S>,
            S: Dialect,
        {
            let wrapper = Wrapper::new(request);
            let id = wrapper.id().to_string();
            let wrapped_request = wrapper.into_json();

            tracing::debug!("Outgoing request: {}", wrapped_request);

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

            R::Response::from_string(response).map(Into::into)
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
            response_rx.recv().await.ok_or_else(|| {
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
            response_rx.recv().await.ok_or_else(|| {
                Error::client_internal("failed to hear back from WebSocket driver".to_string())
            })??;
            Ok(())
        }
    }

    /// Allows us to erase the type signatures associated with the different
    /// WebSocket client variants.
    #[derive(Debug, Clone)]
    pub enum WebSocketClient {
        Unsecure(AsyncTungsteniteClient<Unsecure>),
        Secure(AsyncTungsteniteClient<Secure>),
    }

    impl WebSocketClient {
        pub async fn new_unsecure(
            url: Url,
            compat: CompatMode,
            config: Option<WebSocketConfig>,
        ) -> Result<(Self, WebSocketClientDriver), Error> {
            let (client, driver) =
                AsyncTungsteniteClient::<Unsecure>::new(url, compat, config).await?;
            Ok((Self::Unsecure(client), driver))
        }

        pub async fn new_secure(
            url: Url,
            compat: CompatMode,
            config: Option<WebSocketConfig>,
        ) -> Result<(Self, WebSocketClientDriver), Error> {
            let (client, driver) =
                AsyncTungsteniteClient::<Secure>::new(url, compat, config).await?;
            Ok((Self::Secure(client), driver))
        }

        pub fn close(self) -> Result<(), Error> {
            match self {
                WebSocketClient::Unsecure(c) => c.close(),
                WebSocketClient::Secure(c) => c.close(),
            }
        }
    }

    impl WebSocketClient {
        pub async fn perform<R, S>(&self, request: R, _dialect: S) -> Result<R::Output, Error>
        where
            R: SimpleRequest<S>,
            S: Dialect,
        {
            match self {
                WebSocketClient::Unsecure(c) => c.perform(request).await,
                WebSocketClient::Secure(c) => c.perform(request).await,
            }
        }

        pub async fn subscribe(&self, query: Query) -> Result<Subscription, Error> {
            match self {
                WebSocketClient::Unsecure(c) => c.subscribe(query).await,
                WebSocketClient::Secure(c) => c.subscribe(query).await,
            }
        }

        pub async fn unsubscribe(&self, query: Query) -> Result<(), Error> {
            match self {
                WebSocketClient::Unsecure(c) => c.unsubscribe(query).await,
                WebSocketClient::Secure(c) => c.unsubscribe(query).await,
            }
        }
    }

    use async_tungstenite::tungstenite;

    impl IntoClientRequest for Url {
        fn into_client_request(
            self,
        ) -> tungstenite::Result<tungstenite::handshake::client::Request> {
            let builder = tungstenite::handshake::client::Request::builder()
                .method("GET")
                .header("Host", self.host())
                .header("Connection", "Upgrade")
                .header("Upgrade", "websocket")
                .header("Sec-WebSocket-Version", "13")
                .header(
                    "Sec-WebSocket-Key",
                    tungstenite::handshake::client::generate_key(),
                );

            let builder = if let Some(auth) = authorize(self.as_ref()) {
                builder.header("Authorization", auth.to_string())
            } else {
                builder
            };

            builder
                .uri(self.to_string())
                .body(())
                .map_err(tungstenite::error::Error::HttpFormat)
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

/// Drives the WebSocket connection for a `WebSocketClient` instance.
///
/// This is the primary component responsible for transport-level interaction
/// with the remote WebSocket endpoint.
pub struct WebSocketClientDriver {
    // The underlying WebSocket network connection.
    stream: WebSocketStream<ConnectStream>,
    // Facilitates routing of events to their respective subscriptions.
    router: SubscriptionRouter,
    // How we receive incoming commands from the WebSocketClient.
    cmd_rx: ChannelRx<DriverCommand>,
    // Commands we've received but have not yet completed, indexed by their ID.
    // A Terminate command is executed immediately.
    pending_commands: HashMap<SubscriptionId, DriverCommand>,
    // The compatibility mode directing how to parse subscription events.
    compat: CompatMode,
}

impl WebSocketClientDriver {
    fn new(
        stream: WebSocketStream<ConnectStream>,
        cmd_rx: ChannelRx<DriverCommand>,
        compat: CompatMode,
    ) -> Self {
        Self {
            stream,
            router: SubscriptionRouter::default(),
            cmd_rx,
            pending_commands: HashMap::new(),
            compat,
        }
    }

    async fn send_msg(&mut self, msg: Message) -> Result<(), Error> {
        self.stream.send(msg).await.map_err(|e| {
            Error::web_socket("failed to write to WebSocket connection".to_string(), e)
        })
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

    /// Executes the WebSocket driver, which manages the underlying WebSocket
    /// transport.
    pub async fn run(mut self) -> Result<(), Error> {
        let mut ping_interval =
            tokio::time::interval_at(Instant::now().add(PING_INTERVAL), PING_INTERVAL);

        let recv_timeout = tokio::time::sleep(RECV_TIMEOUT);
        tokio::pin!(recv_timeout);

        loop {
            tokio::select! {
                Some(res) = self.stream.next() => match res {
                    Ok(msg) => {
                        // Reset the receive timeout every time we successfully
                        // receive a message from the remote endpoint.
                        recv_timeout.as_mut().reset(Instant::now().add(RECV_TIMEOUT));
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

    async fn handle_incoming_msg(&mut self, msg: Message) -> Result<(), Error> {
        match msg {
            Message::Text(s) => self.handle_text_msg(s).await,
            Message::Ping(v) => self.pong(v).await,
            _ => Ok(()),
        }
    }

    async fn handle_text_msg(&mut self, msg: String) -> Result<(), Error> {
        let parse_res = match self.compat {
            CompatMode::V0_37 => event::v0_37::DeEvent::from_string(&msg).map(Into::into),
            CompatMode::V0_34 => event::v0_34::DeEvent::from_string(&msg).map(Into::into),
        };
        if let Ok(ev) = parse_res {
            debug!("JSON-RPC event: {}", msg);
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

#[cfg(test)]
mod test {
    use alloc::collections::BTreeMap as HashMap;
    use core::str::FromStr;
    use std::{path::PathBuf, println};

    use async_tungstenite::{
        tokio::{accept_async, TokioAdapter},
        tungstenite::client::IntoClientRequest,
    };
    use futures::StreamExt;
    use http::{header::AUTHORIZATION, Uri};
    use tendermint_config::net;
    use tokio::{
        fs,
        net::{TcpListener, TcpStream},
        task::JoinHandle,
    };

    use super::*;
    use crate::{client::sync::unbounded, event, query::EventType, request, Id, Method};

    // Interface to a driver that manages all incoming WebSocket connections.
    struct TestServer {
        node_addr: net::Address,
        driver_hdl: JoinHandle<Result<(), Error>>,
        terminate_tx: ChannelTx<Result<(), Error>>,
        event_tx: ChannelTx<Event>,
    }

    // A setting telling which of the CometBFT server versions to emulate
    // with the test server.
    #[derive(Copy, Clone)]
    enum TestRpcVersion {
        V0_34,
        V0_37,
        V0_38,
    }

    impl TestServer {
        async fn new(addr: &str, version: TestRpcVersion) -> Self {
            let listener = TcpListener::bind(addr).await.unwrap();
            let local_addr = listener.local_addr().unwrap();
            let node_addr = net::Address::Tcp {
                peer_id: None,
                host: local_addr.ip().to_string(),
                port: local_addr.port(),
            };
            let (terminate_tx, terminate_rx) = unbounded();
            let (event_tx, event_rx) = unbounded();
            let driver = TestServerDriver::new(listener, version, event_rx, terminate_rx);
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
        version: TestRpcVersion,
        event_rx: ChannelRx<Event>,
        terminate_rx: ChannelRx<Result<(), Error>>,
        handlers: Vec<TestServerHandler>,
    }

    impl TestServerDriver {
        fn new(
            listener: TcpListener,
            version: TestRpcVersion,
            event_rx: ChannelRx<Event>,
            terminate_rx: ChannelRx<Result<(), Error>>,
        ) -> Self {
            Self {
                listener,
                version,
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
            self.handlers
                .push(TestServerHandler::new(stream, self.version).await);
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
        async fn new(stream: TcpStream, version: TestRpcVersion) -> Self {
            let conn: WebSocketStream<TokioAdapter<TcpStream>> =
                accept_async(stream).await.unwrap();
            let (terminate_tx, terminate_rx) = unbounded();
            let (event_tx, event_rx) = unbounded();
            let driver = TestServerHandlerDriver::new(conn, version, event_rx, terminate_rx);
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
        version: TestRpcVersion,
        event_rx: ChannelRx<Event>,
        terminate_rx: ChannelRx<Result<(), Error>>,
        // A mapping of subscription queries to subscription IDs for this
        // connection.
        subscriptions: HashMap<String, String>,
    }

    impl TestServerHandlerDriver {
        fn new(
            conn: WebSocketStream<TokioAdapter<TcpStream>>,
            version: TestRpcVersion,
            event_rx: ChannelRx<Event>,
            terminate_rx: ChannelRx<Result<(), Error>>,
        ) -> Self {
            Self {
                conn,
                version,
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
                Some(id) => Id::Str(id.clone()),
                None => return,
            };
            match self.version {
                TestRpcVersion::V0_38 => {
                    let ev: event::v0_38::SerEvent = ev.into();
                    self.send(subs_id, ev).await;
                },
                TestRpcVersion::V0_37 => {
                    let ev: event::v0_37::SerEvent = ev.into();
                    self.send(subs_id, ev).await;
                },
                TestRpcVersion::V0_34 => {
                    let ev: event::v0_34::SerEvent = ev.into();
                    self.send(subs_id, ev).await;
                },
            }
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
                                    "Unexpected method in incoming request: {json_method} ({e})"
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
                println!("Removed subscription {id} for query: {query}");
            }
        }

        async fn send<R>(&mut self, id: Id, res: R)
        where
            R: Serialize,
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

    async fn read_json_fixture(version: &str, name: &str) -> String {
        fs::read_to_string(
            PathBuf::from("./tests/kvstore_fixtures")
                .join(version)
                .join("incoming")
                .join(name.to_owned() + ".json"),
        )
        .await
        .unwrap()
    }

    mod v0_34 {
        use super::*;
        use crate::event::v0_34::DeEvent;

        async fn read_event(name: &str) -> Event {
            DeEvent::from_string(read_json_fixture("v0_34", name).await)
                .unwrap()
                .into()
        }

        #[tokio::test]
        async fn websocket_client_happy_path() {
            let event1 = read_event("subscribe_newblock_0").await;
            let event2 = read_event("subscribe_newblock_1").await;
            let event3 = read_event("subscribe_newblock_2").await;
            let test_events = vec![event1, event2, event3];

            println!("Starting WebSocket server...");
            let mut server = TestServer::new("127.0.0.1:0", TestRpcVersion::V0_34).await;
            println!("Creating client RPC WebSocket connection...");
            let url = server.node_addr.clone().try_into().unwrap();
            let (client, driver) = WebSocketClient::builder(url)
                .compat_mode(CompatMode::V0_34)
                .build()
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

    mod v0_37 {
        use super::*;
        use crate::event::v0_37::DeEvent;

        async fn read_event(name: &str) -> Event {
            DeEvent::from_string(read_json_fixture("v0_37", name).await)
                .unwrap()
                .into()
        }

        #[tokio::test]
        async fn websocket_client_happy_path() {
            let event1 = read_event("subscribe_newblock_0").await;
            let event2 = read_event("subscribe_newblock_1").await;
            let event3 = read_event("subscribe_newblock_2").await;
            let test_events = vec![event1, event2, event3];

            println!("Starting WebSocket server...");
            let mut server = TestServer::new("127.0.0.1:0", TestRpcVersion::V0_37).await;
            println!("Creating client RPC WebSocket connection...");
            let url = server.node_addr.clone().try_into().unwrap();
            let (client, driver) = WebSocketClient::builder(url)
                .compat_mode(CompatMode::V0_37)
                .build()
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

    mod v0_38 {
        use super::*;
        use crate::event::v0_38::DeEvent;

        async fn read_event(name: &str) -> Event {
            DeEvent::from_string(read_json_fixture("v0_38", name).await)
                .unwrap()
                .into()
        }

        #[tokio::test]
        async fn websocket_client_happy_path() {
            let event1 = read_event("subscribe_newblock_0").await;
            let event2 = read_event("subscribe_newblock_1").await;
            let event3 = read_event("subscribe_newblock_2").await;
            let test_events = vec![event1, event2, event3];

            println!("Starting WebSocket server...");
            let mut server = TestServer::new("127.0.0.1:0", TestRpcVersion::V0_38).await;
            println!("Creating client RPC WebSocket connection...");
            let url = server.node_addr.clone().try_into().unwrap();
            let (client, driver) = WebSocketClient::builder(url)
                .compat_mode(CompatMode::V0_37)
                .build()
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

    fn authorization(req: &http::Request<()>) -> Option<&str> {
        req.headers()
            .get(AUTHORIZATION)
            .map(|h| h.to_str().unwrap())
    }

    #[test]
    fn without_basic_auth() {
        let uri = Uri::from_str("http://example.com").unwrap();
        let req = uri.into_client_request().unwrap();

        assert_eq!(authorization(&req), None);
    }

    #[test]
    fn with_basic_auth() {
        let uri = Uri::from_str("http://toto:tata@example.com").unwrap();
        let req = uri.into_client_request().unwrap();

        assert_eq!(authorization(&req), None);
    }
}
