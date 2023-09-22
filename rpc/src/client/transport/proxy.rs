use futures::future::BoxFuture;
use futures::prelude::*;
use futures::ready;
use hyper::client::conn as http_conn;
use hyper::service::Service;
use hyper::upgrade::{self, Upgraded};
use hyper::{Body, Request, StatusCode, Uri};
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{debug, debug_span, Instrument};

use std::boxed::Box;
use std::error::Error;
use std::task::{Context, Poll};

#[derive(Debug)]
pub struct ProxyConnector<C> {
    inner: C,
    proxy_uri: Uri,
}

#[derive(Debug, thiserror::Error)]
enum ConnectError {
    #[error(transparent)]
    Connect(Box<dyn Error + Send + Sync>),
    #[error("HTTP handshake with proxy failed")]
    Handshake(#[source] hyper::Error),
    #[error("HTTP CONNECT request to proxy failed")]
    Request(#[source] hyper::Error),
    #[error("HTTP CONNECT request to proxy was responded with {}", .0)]
    Unsuccessful(StatusCode),
    #[error("HTTP proxy connection upgrade failed")]
    Upgrade(#[source] hyper::Error),
}

impl<C> Service<Uri> for ProxyConnector<C>
where
    C: Service<Uri>,
    C::Response: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    C::Error: Into<Box<dyn Error + Send + Sync>>,
    C::Future: Send,
{
    type Response = Upgraded;

    type Error = ConnectError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.inner.poll_ready(cx)).map_err(|e| ConnectError::Connect(e.into()))?;
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let span = debug_span!("proxy_connect", proxy_uri = %self.proxy_uri, uri = %uri);
        let _entered = span.enter();
        let connect_fut = self.inner.call(self.proxy_uri.clone());
        async move {
            let stream = connect_fut
                .await
                .map_err(|e| ConnectError::Connect(e.into()))?;

            let (mut sender, conn) = http_conn::handshake(stream)
                .await
                .map_err(ConnectError::Handshake)?;
            tokio::task::spawn(async move {
                if let Err(error) = conn.await {
                    debug!(?error, "proxy connection failed");
                }
            });

            let req = Request::connect(uri).body(Body::empty()).unwrap();
            let res = sender
                .send_request(req)
                .await
                .map_err(ConnectError::Request)?;

            // TODO: handle redirects and proxy authentication
            if !res.status().is_success() {
                return Err(ConnectError::Unsuccessful(res.status()));
            }

            upgrade::on(res).await.map_err(ConnectError::Upgrade)
        }
        .instrument(span)
        .boxed()
    }
}
