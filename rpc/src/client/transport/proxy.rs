use futures::future::BoxFuture;
use futures::prelude::*;
use futures::ready;
use hyper::client::conn as http_conn;
use hyper::service::Service;
use hyper::{Body, Request, StatusCode, Uri};
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{debug, debug_span, Instrument};

use std::boxed::Box;
use std::error::Error;
use std::task::{Context, Poll};

#[derive(Clone, Debug)]
pub struct ProxyConnector<C> {
    inner: C,
    proxy_uri: Uri,
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error(transparent)]
    Connect(Box<dyn Error + Send + Sync>),
    #[error("HTTP handshake with proxy failed")]
    Handshake(#[source] hyper::Error),
    #[error("HTTP CONNECT request to proxy failed")]
    Request(#[source] hyper::Error),
    #[error("HTTP CONNECT request to proxy was responded with {}", .0)]
    Unsuccessful(StatusCode),
    #[error("HTTP proxy connection failed")]
    Connection(#[source] hyper::Error),
    #[error("unexpected data after HTTP CONNECT response")]
    UnexpectedResponseData,
}

impl<C> ProxyConnector<C> {
    pub fn new(connector: C, proxy_uri: Uri) -> Self {
        ProxyConnector {
            inner: connector,
            proxy_uri,
        }
    }
}

impl<C> Service<Uri> for ProxyConnector<C>
where
    C: Service<Uri>,
    C::Response: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    C::Error: Into<Box<dyn Error + Send + Sync>>,
    C::Future: Send + 'static,
{
    type Response = C::Response;

    type Error = ConnectError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.inner.poll_ready(cx)).map_err(|e| ConnectError::Connect(e.into()))?;
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let span = debug_span!("proxy_connect", proxy_uri = %self.proxy_uri, uri = %uri);
        let connect_fut = {
            let _entered = span.enter();
            self.inner.call(self.proxy_uri.clone())
        };
        async move {
            let io = connect_fut
                .await
                .map_err(|e| ConnectError::Connect(e.into()))?;

            let (mut sender, conn) = http_conn::handshake(io)
                .await
                .map_err(ConnectError::Handshake)?;
            let conn_task = tokio::task::spawn(conn.without_shutdown());

            let req = Request::connect(uri).body(Body::empty()).unwrap();
            let res = sender
                .send_request(req)
                .await
                .map_err(ConnectError::Request)?;

            // TODO: handle redirects and proxy authentication
            if !res.status().is_success() {
                return Err(ConnectError::Unsuccessful(res.status()));
            }

            let http_conn::Parts { io, read_buf, .. } =
                conn_task.await.unwrap().map_err(ConnectError::Connection)?;

            // There should not be any initial bytes from the proxy or the
            // destination server. With TLS, the handshake must start with a
            // client hello. With plaintext HTTP, the server should expect the
            // client to send either an HTTP/1.1 request with a possible Upgrade
            // header, or an HTTP/2 preface. It should be very uncommon for the
            // server to fire away a HTTP/2 SETTINGS frame without prior
            // negotiation, even though RFC 7540 does not expressly forbid this.
            // Assuming this, we can treat any pre-received data as an error
            // and return the underlying connection object.
            if !read_buf.is_empty() {
                debug!("unexpected bytes in CONNECT response: {:?}", read_buf);
                return Err(ConnectError::UnexpectedResponseData);
            }

            Ok(io)
        }
        .instrument(span)
        .boxed()
    }
}
