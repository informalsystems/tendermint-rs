//! HTTP-based transport for Tendermint RPC Client, with WebSockets-based
//! subscription handling mechanism.

use async_trait::async_trait;
use hyper::header;
use tendermint::net;

use crate::{client::transport::Transport, Error};
use bytes::buf::BufExt;
use std::io::Read;

#[derive(Debug)]
pub struct HttpWsTransport {
    host: String,
    port: u16,
}

#[async_trait]
impl Transport for HttpWsTransport {
    async fn request(&self, request_body: String) -> Result<String, Error> {
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
        let mut response_string = String::new();
        let _ = response_body
            .reader()
            .read_to_string(&mut response_string)
            .map_err(|e| Error::internal_error(format!("failed to read response body: {}", e)));
        Ok(response_string)
    }
}

impl HttpWsTransport {
    /// Create a new HTTP/WebSockets
    pub fn new(address: net::Address) -> Result<Self, Error> {
        let (host, port) = match address {
            net::Address::Tcp { host, port, .. } => (host, port),
            other => {
                return Err(Error::invalid_params(&format!(
                    "invalid RPC address: {:?}",
                    other
                )))
            }
        };
        Ok(HttpWsTransport { host, port })
    }
}
