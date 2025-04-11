//! HTTP-based transport for Tendermint RPC Client.

use core::str::FromStr;
use core::time::Duration;

use async_trait::async_trait;
use reqwest::{header, Proxy};

use tendermint::{block::Height, evidence::Evidence, Hash};
use tendermint_config::net;

use crate::client::{Client, CompatMode};
use crate::dialect::{v0_34, v0_37, v0_38, Dialect, LatestDialect};
use crate::endpoint;
use crate::prelude::*;
use crate::query::Query;
use crate::request::RequestMessage;
use crate::response::Response;
use crate::{Error, Order, Scheme, SimpleRequest, Url};

use super::auth;

const USER_AGENT: &str = concat!("tendermint.rs/", env!("CARGO_PKG_VERSION"));

/// `reqwest` http client re-exported for compatibility convenience
pub type ReqwestClient = reqwest::Client;

/// A JSON-RPC/HTTP Tendermint RPC client (implements [`crate::Client`]).
///
/// Supports both HTTP and HTTPS connections to Tendermint RPC endpoints, and
/// allows for the use of HTTP proxies (see [`HttpClient::new_with_proxy`] for
/// details).
///
/// Does not provide [`crate::event::Event`] subscription facilities (see
/// [`crate::WebSocketClient`] for a client that does).
///
/// ## Examples
///
/// ```rust,ignore
/// use tendermint_rpc::{HttpClient, Client};
///
/// #[tokio::main]
/// async fn main() {
///     let client = HttpClient::new("http://127.0.0.1:26657")
///         .unwrap();
///
///     let abci_info = client.abci_info()
///         .await
///         .unwrap();
///
///     println!("Got ABCI info: {:?}", abci_info);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct HttpClient {
    inner: ReqwestClient,
    url: reqwest::Url,
    compat: CompatMode,
}

/// The builder pattern constructor for [`HttpClient`].
pub struct Builder {
    url: HttpClientUrl,
    compat: CompatMode,
    proxy_url: Option<HttpClientUrl>,
    user_agent: Option<String>,
    timeout: Duration,
    client: Option<ReqwestClient>,
}

impl Builder {
    /// Use the specified compatibility mode for the Tendermint RPC protocol.
    ///
    /// The default is the latest protocol version supported by this crate.
    pub fn compat_mode(mut self, mode: CompatMode) -> Self {
        self.compat = mode;
        self
    }

    /// Specify the URL of a proxy server for the client to connect through.
    ///
    /// If the RPC endpoint is secured (HTTPS), the proxy will automatically
    /// attempt to connect using the [HTTP CONNECT] method.
    ///
    /// [HTTP CONNECT]: https://en.wikipedia.org/wiki/HTTP_tunnel
    pub fn proxy_url(mut self, url: HttpClientUrl) -> Self {
        self.proxy_url = Some(url);
        self
    }

    /// The timeout is applied from when the request starts connecting until
    /// the response body has finished.
    ///
    /// The default is 30 seconds.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }

    /// Specify the custom User-Agent header used by the client.
    pub fn user_agent(mut self, agent: String) -> Self {
        self.user_agent = Some(agent);
        self
    }

    /// Use the provided client instead of building one internally.
    ///
    /// ## Warning
    /// This will override the following options set on the builder:
    /// `timeout`, `user_agent`, and `proxy_url`.
    pub fn client(mut self, client: ReqwestClient) -> Self {
        self.client = Some(client);
        self
    }

    /// Try to create a client with the options specified for this builder.
    pub fn build(self) -> Result<HttpClient, Error> {
        let inner = if let Some(inner) = self.client {
            inner
        } else {
            let builder = reqwest::ClientBuilder::new()
                .user_agent(self.user_agent.unwrap_or_else(|| USER_AGENT.to_string()))
                .timeout(self.timeout);

            match self.proxy_url {
                None => builder.build().map_err(Error::http)?,
                Some(proxy_url) => {
                    let proxy = if self.url.0.is_secure() {
                        Proxy::https(reqwest::Url::from(proxy_url.0))
                            .map_err(Error::invalid_proxy)?
                    } else {
                        Proxy::http(reqwest::Url::from(proxy_url.0))
                            .map_err(Error::invalid_proxy)?
                    };
                    builder.proxy(proxy).build().map_err(Error::http)?
                },
            }
        };

        Ok(HttpClient {
            inner,
            url: self.url.into(),
            compat: self.compat,
        })
    }
}

impl HttpClient {
    /// Construct a new Tendermint RPC HTTP/S client connecting to the given
    /// URL. This avoids using the `Builder` and thus does not perform any
    /// validation of the configuration.
    pub fn new_from_parts(inner: ReqwestClient, url: reqwest::Url, compat: CompatMode) -> Self {
        Self { inner, url, compat }
    }

    /// Construct a new Tendermint RPC HTTP/S client connecting to the given
    /// URL.
    pub fn new<U>(url: U) -> Result<Self, Error>
    where
        U: TryInto<HttpClientUrl, Error = Error>,
    {
        let url = url.try_into()?;
        Self::builder(url).build()
    }

    /// Construct a new Tendermint RPC HTTP/S client connecting to the given
    /// URL, but via the specified proxy's URL.
    ///
    /// If the RPC endpoint is secured (HTTPS), the proxy will automatically
    /// attempt to connect using the [HTTP CONNECT] method.
    ///
    /// [HTTP CONNECT]: https://en.wikipedia.org/wiki/HTTP_tunnel
    pub fn new_with_proxy<U, P>(url: U, proxy_url: P) -> Result<Self, Error>
    where
        U: TryInto<HttpClientUrl, Error = Error>,
        P: TryInto<HttpClientUrl, Error = Error>,
    {
        let url = url.try_into()?;
        Self::builder(url).proxy_url(proxy_url.try_into()?).build()
    }

    /// Initiate a builder for a Tendermint RPC HTTP/S client connecting
    /// to the given URL, so that more configuration options can be specified
    /// with the builder.
    pub fn builder(url: HttpClientUrl) -> Builder {
        Builder {
            url,
            compat: Default::default(),
            proxy_url: None,
            user_agent: None,
            timeout: Duration::from_secs(30),
            client: None,
        }
    }

    /// Set compatibility mode on the instantiated client.
    ///
    /// As the HTTP client is stateless and does not support subscriptions,
    /// the protocol version it uses can be changed at will, for example,
    /// as a result of version discovery over the `/status` endpoint.
    pub fn set_compat_mode(&mut self, compat: CompatMode) {
        self.compat = compat;
    }

    fn build_request<R>(&self, request: R) -> Result<reqwest::Request, Error>
    where
        R: RequestMessage,
    {
        let request_body = request.into_json();

        tracing::debug!(url = %self.url, body = %request_body, "outgoing request");

        let mut builder = self
            .inner
            .post(auth::strip_authority(self.url.clone()))
            .header(header::CONTENT_TYPE, "application/json")
            .body(request_body.into_bytes());

        if let Some(auth) = auth::authorize(&self.url) {
            builder = builder.header(header::AUTHORIZATION, auth.to_string());
        }

        builder.build().map_err(Error::http)
    }

    async fn perform_with_dialect<R, S>(&self, request: R, _dialect: S) -> Result<R::Output, Error>
    where
        R: SimpleRequest<S>,
        S: Dialect,
    {
        let request = self.build_request(request)?;
        let response = self.inner.execute(request).await.map_err(Error::http)?;
        let response_status = response.status();
        let response_body = response.bytes().await.map_err(Error::http)?;

        tracing::debug!(
            status = %response_status,
            body = %String::from_utf8_lossy(&response_body),
            "incoming response"
        );

        // Successful JSON-RPC requests are expected to return a 200 OK HTTP status.
        // Otherwise, this means that the HTTP request failed as a whole,
        // as opposed to the JSON-RPC request returning an error,
        // and we cannot expect the response body to be a valid JSON-RPC response.
        if response_status != reqwest::StatusCode::OK {
            return Err(Error::http_request_failed(response_status));
        }

        R::Response::from_string(&response_body).map(Into::into)
    }
}

#[async_trait]
impl Client for HttpClient {
    async fn perform<R>(&self, request: R) -> Result<R::Output, Error>
    where
        R: SimpleRequest,
    {
        self.perform_with_dialect(request, LatestDialect).await
    }

    async fn block<H>(&self, height: H) -> Result<endpoint::block::Response, Error>
    where
        H: Into<Height> + Send,
    {
        perform_with_compat!(self, endpoint::block::Request::new(height.into()))
    }

    async fn block_by_hash(
        &self,
        hash: tendermint::Hash,
    ) -> Result<endpoint::block_by_hash::Response, Error> {
        perform_with_compat!(self, endpoint::block_by_hash::Request::new(hash))
    }

    async fn latest_block(&self) -> Result<endpoint::block::Response, Error> {
        perform_with_compat!(self, endpoint::block::Request::default())
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

    async fn block_search(
        &self,
        query: Query,
        page: u32,
        per_page: u8,
        order: Order,
    ) -> Result<endpoint::block_search::Response, Error> {
        perform_with_compat!(
            self,
            endpoint::block_search::Request::new(query, page, per_page, order)
        )
    }

    async fn header<H>(&self, height: H) -> Result<endpoint::header::Response, Error>
    where
        H: Into<Height> + Send,
    {
        let height = height.into();
        match self.compat {
            CompatMode::V0_38 => {
                self.perform_with_dialect(endpoint::header::Request::new(height), v0_38::Dialect)
                    .await
            },
            CompatMode::V0_37 => {
                self.perform_with_dialect(endpoint::header::Request::new(height), v0_37::Dialect)
                    .await
            },
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
            CompatMode::V0_38 => {
                self.perform_with_dialect(
                    endpoint::header_by_hash::Request::new(hash),
                    v0_38::Dialect,
                )
                .await
            },
            CompatMode::V0_37 => {
                self.perform_with_dialect(
                    endpoint::header_by_hash::Request::new(hash),
                    v0_37::Dialect,
                )
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

    /// `/broadcast_evidence`: broadcast an evidence.
    async fn broadcast_evidence(
        &self,
        evidence: Evidence,
    ) -> Result<endpoint::evidence::Response, Error> {
        match self.compat {
            CompatMode::V0_38 => {
                let request = endpoint::evidence::Request::new(evidence);
                self.perform_with_dialect(request, crate::dialect::v0_38::Dialect)
                    .await
            },
            CompatMode::V0_37 => {
                let request = endpoint::evidence::Request::new(evidence);
                self.perform_with_dialect(request, crate::dialect::v0_37::Dialect)
                    .await
            },
            CompatMode::V0_34 => {
                let request = endpoint::evidence::Request::new(evidence);
                self.perform_with_dialect(request, crate::dialect::v0_34::Dialect)
                    .await
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

/// A URL limited to use with HTTP clients.
///
/// Facilitates useful type conversions and inferences.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HttpClientUrl(Url);

impl TryFrom<Url> for HttpClientUrl {
    type Error = Error;

    fn try_from(value: Url) -> Result<Self, Error> {
        match value.scheme() {
            Scheme::Http | Scheme::Https => Ok(Self(value)),
            _ => Err(Error::invalid_url(value)),
        }
    }
}

impl FromStr for HttpClientUrl {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let url: Url = s.parse()?;
        url.try_into()
    }
}

impl TryFrom<&str> for HttpClientUrl {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Error> {
        value.parse()
    }
}

impl TryFrom<net::Address> for HttpClientUrl {
    type Error = Error;

    fn try_from(value: net::Address) -> Result<Self, Error> {
        match value {
            net::Address::Tcp {
                peer_id: _,
                host,
                port,
            } => format!("http://{host}:{port}").parse(),
            net::Address::Unix { .. } => Err(Error::invalid_network_address()),
        }
    }
}

impl From<HttpClientUrl> for Url {
    fn from(url: HttpClientUrl) -> Self {
        url.0
    }
}

impl From<HttpClientUrl> for url::Url {
    fn from(url: HttpClientUrl) -> Self {
        url.0.into()
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use reqwest::{header::AUTHORIZATION, Request};

    use super::HttpClient;
    use crate::endpoint::abci_info;
    use crate::Url;

    fn authorization(req: &Request) -> Option<&str> {
        req.headers()
            .get(AUTHORIZATION)
            .map(|h| h.to_str().unwrap())
    }

    #[test]
    fn without_basic_auth() {
        let url = Url::from_str("http://example.com").unwrap();
        let client = HttpClient::new(url).unwrap();
        let req = HttpClient::build_request(&client, abci_info::Request).unwrap();

        assert_eq!(authorization(&req), None);
    }

    #[test]
    fn with_basic_auth() {
        let url = Url::from_str("http://toto:tata@example.com").unwrap();
        let client = HttpClient::new(url).unwrap();
        let req = HttpClient::build_request(&client, abci_info::Request).unwrap();

        assert_eq!(authorization(&req), Some("Basic dG90bzp0YXRh"));
        let num_auth_headers = req
            .headers()
            .iter()
            .filter(|h| h.0 == AUTHORIZATION)
            .count();
        assert_eq!(num_auth_headers, 1);
    }
}
