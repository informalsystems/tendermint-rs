//! HTTP-based transport for Tendermint RPC Client.

use async_trait::async_trait;
use hyper::body::Buf;
use hyper::{header, Uri};

use tendermint::net;

use crate::client::transport::utils::get_tcp_host_port;
use crate::{Client, Error, Response, Result, SimpleRequest};
use hyper::client::connect::Connect;
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use std::convert::TryInto;
use std::io::Read;

/// A JSON-RPC/HTTP Tendermint RPC client (implements [`Client`]).
///
/// Does not provide [`crate::event::Event`] subscription facilities (see
/// [`crate::WebSocketClient`] for a client that does).
///
/// Does not provide any security. For a JSON-RPC/HTTPS client, see
/// [`HttpsClient`].
///
/// ## Examples
///
/// ```rust,ignore
/// use tendermint_rpc::{HttpClient, Client};
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
pub type HttpClient = HyperClient<HttpConnector>;

/// A JSON-RPC/HTTPS (i.e. HTTP/TLS) Tendermint RPC client.
///
/// Similar to [`HttpClient`], but allows for connection to the RPC endpoint
/// via HTTPS.
pub type HttpsClient = HyperClient<HttpsConnector<HttpConnector>>;

/// A [`hyper`]-based Tendermint RPC client.
///
/// Generic over the connector type used for the client.
///
/// [`hyper`]: https://hyper.rs/
#[derive(Debug, Clone)]
pub struct HyperClient<C> {
    uri: Uri,
    inner: hyper::Client<C, hyper::Body>,
}

#[async_trait]
impl<C> Client for HyperClient<C>
where
    C: Connect + Clone + Send + Sync + 'static,
{
    async fn perform<R>(&self, request: R) -> Result<R::Response>
    where
        R: SimpleRequest,
    {
        let request = self.build_request(request)?;
        let response = self.inner.request(request).await?;
        let response_body = response_to_string(response).await?;
        tracing::debug!("Incoming response: {}", response_body);
        R::Response::from_string(&response_body)
    }
}

impl HyperClient<HttpConnector> {
    /// Create a new JSON-RPC/HTTP Tendermint RPC client.
    pub fn new(address: net::Address) -> Result<Self> {
        let (host, port) = get_tcp_host_port(address)?;
        Ok(Self {
            uri: format!("http://{}:{}/", host, port).try_into()?,
            inner: hyper::Client::new(),
        })
    }
}

impl HyperClient<HttpsConnector<HttpConnector>> {
    /// Create a new JSON-RPC/HTTPS (i.e. HTTP/TLS) Tendermint RPC client.
    pub fn new(address: net::Address) -> Result<Self> {
        let (host, port) = get_tcp_host_port(address)?;
        Ok(Self {
            uri: format!("https://{}:{}/", host, port).try_into()?,
            inner: hyper::Client::builder().build(HttpsConnector::with_native_roots()),
        })
    }
}

impl<C> HyperClient<C> {
    fn build_request<R: SimpleRequest>(&self, request: R) -> Result<hyper::Request<hyper::Body>> {
        let request_body = request.into_json();

        let mut request = hyper::Request::builder()
            .method("POST")
            .uri(&self.uri)
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

        Ok(request)
    }
}

async fn response_to_string(response: hyper::Response<hyper::Body>) -> Result<String> {
    let mut response_body = String::new();
    hyper::body::aggregate(response.into_body())
        .await?
        .reader()
        .read_to_string(&mut response_body)
        .map_err(|_| Error::client_internal_error("failed to read response body to string"))?;
    Ok(response_body)
}
