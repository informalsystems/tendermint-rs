//! HTTP-based transport for Tendermint RPC Client.

use crate::client::transport::utils::get_tcp_host_port;
use crate::{Client, Response, Result, SimpleRequest};
use async_trait::async_trait;
use bytes::buf::BufExt;
use hyper::header;
use tendermint::net;

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
    async fn perform<R>(&mut self, request: R) -> Result<R::Response>
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
        let http_client = hyper::Client::builder().build_http();
        let response = http_client.request(request).await?;
        let response_body = hyper::body::aggregate(response.into_body()).await?;
        R::Response::from_reader(response_body.reader())
    }
}

impl HttpClient {
    /// Create a new JSON-RPC/HTTP Tendermint RPC client.
    pub fn new(address: net::Address) -> Result<Self> {
        let (host, port) = get_tcp_host_port(address)?;
        Ok(HttpClient { host, port })
    }
}
