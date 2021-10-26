//! HTTP-based transport for Tendermint RPC Client.

use crate::client::Client;
use crate::prelude::*;
use crate::{Error, Scheme, SimpleRequest, Url};
use async_trait::async_trait;
use core::convert::{TryFrom, TryInto};
use core::str::FromStr;
use tendermint_config::net;

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
        let proxy_url = proxy_url.try_into()?;
        Ok(Self {
            inner: if proxy_url.0.is_secure() {
                sealed::HttpClient::new_https_proxy(url.try_into()?, proxy_url.try_into()?)?
            } else {
                sealed::HttpClient::new_http_proxy(url.try_into()?, proxy_url.try_into()?)?
            },
        })
    }
}

#[async_trait]
impl Client for HttpClient {
    async fn perform<R>(&self, request: R) -> Result<R::Response, Error>
    where
        R: SimpleRequest,
    {
        self.inner.perform(request).await
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
            } => format!("http://{}:{}", host, port).parse(),
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
    use crate::prelude::*;
    use crate::{Error, Response, SimpleRequest};
    use hyper::body::Buf;
    use hyper::client::connect::Connect;
    use hyper::client::HttpConnector;
    use hyper::{header, Uri};
    use hyper_proxy::{Intercept, Proxy, ProxyConnector};
    use hyper_rustls::HttpsConnector;
    use std::io::Read;

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
        pub async fn perform<R>(&self, request: R) -> Result<R::Response, Error>
        where
            R: SimpleRequest,
        {
            let request = self.build_request(request)?;
            let response = self.inner.request(request).await.map_err(Error::hyper)?;
            let response_body = response_to_string(response).await?;
            tracing::debug!("Incoming response: {}", response_body);
            R::Response::from_string(&response_body)
        }
    }

    impl<C> HyperClient<C> {
        /// Build a request using the given Tendermint RPC request.
        pub fn build_request<R: SimpleRequest>(
            &self,
            request: R,
        ) -> Result<hyper::Request<hyper::Body>, Error> {
            let request_body = request.into_json();

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

        pub async fn perform<R>(&self, request: R) -> Result<R::Response, Error>
        where
            R: SimpleRequest,
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
