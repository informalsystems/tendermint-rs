//! HTTP-based transport for Tendermint RPC Client.

use async_trait::async_trait;
use hyper::body::Buf;
use hyper::header;

use tendermint::net;

use crate::client::transport::utils::get_tcp_host_port;
use crate::{Client, Error, Response, Result, SimpleRequest};
use std::io::Read;

/// A JSON-RPC/HTTP Tendermint RPC client (implements [`Client`]).
///
/// Does not provide [`Event`] subscription facilities (see [`WebSocketClient`]
/// for a client that does provide [`Event`] subscription facilities).
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
///
/// [`Client`]: trait.Client.html
/// [`Event`]: ./event/struct.Event.html
/// [`WebSocketClient`]: struct.WebSocketClient.html
#[derive(Debug, Clone)]
pub struct HttpClient {
    host: String,
    port: u16,
}

#[async_trait]
impl Client for HttpClient {
    async fn perform<R>(&self, request: R) -> Result<R::Response>
    where
        R: SimpleRequest,
    {
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

        let http_client = hyper::Client::new();
        let response = http_client.request(request).await?;
        let mut response_body = String::new();
        hyper::body::aggregate(response.into_body())
            .await?
            .reader()
            .read_to_string(&mut response_body)
            .map_err(|_| Error::client_internal_error("failed to read response body to string"))?;
        tracing::debug!("Incoming response: {}", response_body);
        R::Response::from_string(&response_body)
    }
}

impl HttpClient {
    /// Create a new JSON-RPC/HTTP Tendermint RPC client.
    pub fn new(address: net::Address) -> Result<Self> {
        let (host, port) = get_tcp_host_port(address)?;
        Ok(HttpClient { host, port })
    }
}
