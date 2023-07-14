//! HTTP-based transport for Tendermint RPC Client.

use core::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};

use async_trait::async_trait;

use tendermint::{block::Height, evidence::Evidence, Hash};
use tendermint_config::net;

use crate::prelude::*;
use crate::{
    client::{Client, CompatMode},
    dialect, endpoint,
    query::Query,
    Error, Order, Scheme, SimpleRequest, Url,
};

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
    inner: sealed::HttpClient,
    compat: CompatMode,
}

/// The builder pattern constructor for [`HttpClient`].
pub struct Builder {
    url: HttpClientUrl,
    compat: CompatMode,
    proxy_url: Option<HttpClientUrl>,
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

    /// Try to create a client with the options specified for this builder.
    pub fn build(self) -> Result<HttpClient, Error> {
        match self.proxy_url {
            None => Ok(HttpClient {
                inner: if self.url.0.is_secure() {
                    sealed::HttpClient::new_https(self.url.try_into()?)
                } else {
                    sealed::HttpClient::new_http(self.url.try_into()?)
                },
                compat: self.compat,
            }),
            Some(proxy_url) => Ok(HttpClient {
                inner: if proxy_url.0.is_secure() {
                    sealed::HttpClient::new_https_proxy(
                        self.url.try_into()?,
                        proxy_url.try_into()?,
                    )?
                } else {
                    sealed::HttpClient::new_http_proxy(self.url.try_into()?, proxy_url.try_into()?)?
                },
                compat: self.compat,
            }),
        }
    }
}

impl HttpClient {
    /// Construct a new Tendermint RPC HTTP/S client connecting to the given
    /// URL.
    pub fn new<U>(url: U) -> Result<Self, Error>
    where
        U: TryInto<HttpClientUrl, Error = Error>,
    {
        let url = url.try_into()?;
        Ok(Self {
            inner: if url.0.is_secure() {
                sealed::HttpClient::new_https(url.try_into()?)
            } else {
                sealed::HttpClient::new_http(url.try_into()?)
            },
            compat: Default::default(),
        })
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

    async fn perform_v0_34<R>(&self, request: R) -> Result<R::Output, Error>
    where
        R: SimpleRequest<dialect::v0_34::Dialect>,
    {
        self.inner.perform(request).await
    }
}

#[async_trait]
impl Client for HttpClient {
    async fn perform<R>(&self, request: R) -> Result<R::Output, Error>
    where
        R: SimpleRequest,
    {
        self.inner.perform(request).await
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
                    .perform_v0_34(endpoint::block::Request::new(height))
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
                    .perform_v0_34(endpoint::block_by_hash::Request::new(hash))
                    .await?;
                Ok(resp.into())
            },
        }
    }

    /// `/broadcast_evidence`: broadcast an evidence.
    async fn broadcast_evidence(&self, e: Evidence) -> Result<endpoint::evidence::Response, Error> {
        match self.compat {
            CompatMode::V0_37 => self.perform(endpoint::evidence::Request::new(e)).await,
            CompatMode::V0_34 => {
                self.perform_v0_34(endpoint::evidence::Request::new(e))
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

impl TryFrom<HttpClientUrl> for hyper::Uri {
    type Error = Error;

    fn try_from(value: HttpClientUrl) -> Result<Self, Error> {
        value.0.to_string().parse().map_err(Error::invalid_uri)
    }
}

mod sealed {
    use std::io::Read;

    use http::header::AUTHORIZATION;
    use hyper::{
        body::Buf,
        client::{connect::Connect, HttpConnector},
        header, Uri,
    };
    use hyper_proxy::{Intercept, Proxy, ProxyConnector};
    use hyper_rustls::HttpsConnector;

    use crate::prelude::*;
    use crate::{
        client::transport::auth::authorize, dialect::Dialect, Error, Response, SimpleRequest,
    };

    /// A wrapper for a `hyper`-based client, generic over the connector type.
    #[derive(Debug, Clone)]
    pub struct HyperClient<C> {
        uri: Uri,
        inner: hyper::Client<C>,
    }

    impl<C> HyperClient<C> {
        pub fn new(uri: Uri, inner: hyper::Client<C>) -> Self {
            Self { uri, inner }
        }
    }

    impl<C> HyperClient<C>
    where
        C: Connect + Clone + Send + Sync + 'static,
    {
        pub async fn perform<R, S>(&self, request: R) -> Result<R::Output, Error>
        where
            R: SimpleRequest<S>,
            S: Dialect,
        {
            let request = self.build_request(request)?;
            let response = self.inner.request(request).await.map_err(Error::hyper)?;
            let response_body = response_to_string(response).await?;
            tracing::debug!("Incoming response: {}", response_body);
            R::Response::from_string(&response_body).map(Into::into)
        }
    }

    impl<C> HyperClient<C> {
        /// Build a request using the given Tendermint RPC request.
        pub fn build_request<R, S>(&self, request: R) -> Result<hyper::Request<hyper::Body>, Error>
        where
            R: SimpleRequest<S>,
            S: Dialect,
        {
            let request_body = request.into_json();

            tracing::debug!("Outgoing request: {}", request_body);

            let mut request = hyper::Request::builder()
                .method("POST")
                .uri(&self.uri)
                .body(hyper::Body::from(request_body.into_bytes()))
                .map_err(Error::http)?;

            {
                let headers = request.headers_mut();
                headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
                headers.insert(
                    header::USER_AGENT,
                    format!("tendermint.rs/{}", env!("CARGO_PKG_VERSION"))
                        .parse()
                        .unwrap(),
                );

                if let Some(auth) = authorize(&self.uri) {
                    headers.insert(AUTHORIZATION, auth.to_string().parse().unwrap());
                }
            }

            Ok(request)
        }
    }

    /// We offer several variations of `hyper`-based client.
    ///
    /// Here we erase the type signature of the underlying `hyper`-based
    /// client, allowing the higher-level HTTP client to operate via HTTP or
    /// HTTPS, and with or without a proxy.
    #[derive(Debug, Clone)]
    pub enum HttpClient {
        Http(HyperClient<HttpConnector>),
        Https(HyperClient<HttpsConnector<HttpConnector>>),
        HttpProxy(HyperClient<ProxyConnector<HttpConnector>>),
        HttpsProxy(HyperClient<ProxyConnector<HttpsConnector<HttpConnector>>>),
    }

    impl HttpClient {
        pub fn new_http(uri: Uri) -> Self {
            Self::Http(HyperClient::new(uri, hyper::Client::new()))
        }

        pub fn new_https(uri: Uri) -> Self {
            Self::Https(HyperClient::new(
                uri,
                hyper::Client::builder().build(HttpsConnector::with_native_roots()),
            ))
        }

        pub fn new_http_proxy(uri: Uri, proxy_uri: Uri) -> Result<Self, Error> {
            let proxy = Proxy::new(Intercept::All, proxy_uri);
            let proxy_connector =
                ProxyConnector::from_proxy(HttpConnector::new(), proxy).map_err(Error::io)?;
            Ok(Self::HttpProxy(HyperClient::new(
                uri,
                hyper::Client::builder().build(proxy_connector),
            )))
        }

        pub fn new_https_proxy(uri: Uri, proxy_uri: Uri) -> Result<Self, Error> {
            let proxy = Proxy::new(Intercept::All, proxy_uri);
            let proxy_connector =
                ProxyConnector::from_proxy(HttpsConnector::with_native_roots(), proxy)
                    .map_err(Error::io)?;

            Ok(Self::HttpsProxy(HyperClient::new(
                uri,
                hyper::Client::builder().build(proxy_connector),
            )))
        }

        pub async fn perform<R, S>(&self, request: R) -> Result<R::Output, Error>
        where
            R: SimpleRequest<S>,
            S: Dialect,
        {
            match self {
                HttpClient::Http(c) => c.perform(request).await,
                HttpClient::Https(c) => c.perform(request).await,
                HttpClient::HttpProxy(c) => c.perform(request).await,
                HttpClient::HttpsProxy(c) => c.perform(request).await,
            }
        }
    }

    async fn response_to_string(response: hyper::Response<hyper::Body>) -> Result<String, Error> {
        let mut response_body = String::new();
        hyper::body::aggregate(response.into_body())
            .await
            .map_err(Error::hyper)?
            .reader()
            .read_to_string(&mut response_body)
            .map_err(Error::io)?;

        Ok(response_body)
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use http::{header::AUTHORIZATION, Request, Uri};
    use hyper::Body;

    use super::sealed::HyperClient;
    use crate::dialect::LatestDialect;
    use crate::endpoint::abci_info;

    fn authorization(req: &Request<Body>) -> Option<&str> {
        req.headers()
            .get(AUTHORIZATION)
            .map(|h| h.to_str().unwrap())
    }

    #[test]
    fn without_basic_auth() {
        let uri = Uri::from_str("http://example.com").unwrap();
        let inner = hyper::Client::new();
        let client = HyperClient::new(uri, inner);
        let req =
            HyperClient::build_request::<_, LatestDialect>(&client, abci_info::Request).unwrap();

        assert_eq!(authorization(&req), None);
    }

    #[test]
    fn with_basic_auth() {
        let uri = Uri::from_str("http://toto:tata@example.com").unwrap();
        let inner = hyper::Client::new();
        let client = HyperClient::new(uri, inner);
        let req =
            HyperClient::build_request::<_, LatestDialect>(&client, abci_info::Request).unwrap();

        assert_eq!(authorization(&req), Some("Basic dG90bzp0YXRh"));
    }
}
