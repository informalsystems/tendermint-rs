use futures::future::BoxFuture;
use futures::prelude::*;
use futures::ready;
use http::uri::Scheme;
use hyper::client::connect::dns::{self, GaiFuture as ResolveFuture, GaiResolver};
use hyper::service::Service;
use hyper::upgrade::Upgraded;
use hyper::Uri;
use tracing::{debug, debug_span, Instrument};

use std::io;
use std::task::{Context, Poll};

pub struct ProxyConnector {
    proxy_uri: Uri,
    resolver: GaiResolver,
}

#[derive(Debug, thiserror::Error)]
enum ConnectError {
    #[error(transparent)]
    Dns(#[from] io::Error),
    #[error("no scheme in proxy URI")]
    UriSchemeMissing,
    #[error("proxy URI scheme {} not supported", .0)]
    UriSchemeNotSupported(Scheme),
    #[error("no host part in proxy URI")]
    UriHostMissing,
    #[error("invalid host part in proxy URI")]
    UriHostInvalid(#[from] dns::InvalidNameError),
}

impl Service<Uri> for ProxyConnector {
    type Response = Upgraded;

    type Error = ConnectError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.resolver.poll_ready(cx))?;
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let span = debug_span!("proxy_connect", proxy_uri = %self.proxy_uri, uri = %uri);
        let _entered = span.enter();
        let resolve_res = self.resolve();
        async move {
            let sockaddrs = resolve_res?.await?;
            for sockaddr in sockaddrs {
                debug!("connecting to {:?}", sockaddr);
            }
            todo!()
        }
        .instrument(span)
        .boxed()
    }
}

impl ProxyConnector {
    fn resolve(&mut self) -> Result<ResolveFuture, ConnectError> {
        let scheme = self
            .proxy_uri
            .scheme()
            .ok_or(ConnectError::UriSchemeMissing)?;
        if scheme != &Scheme::HTTP && scheme != &Scheme::HTTPS {
            return Err(ConnectError::UriSchemeNotSupported(scheme.clone()));
        }
        let hostname = self
            .proxy_uri
            .host()
            .ok_or(ConnectError::UriHostMissing)?
            .parse::<dns::Name>()?;
        debug!("resolving proxy host {:?}", hostname);
        Ok(self.resolver.call(hostname))
    }
}
