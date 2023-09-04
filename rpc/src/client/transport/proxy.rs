use futures::future::BoxFuture;
use hyper::service::Service;
use hyper::upgrade::Upgraded;
use hyper::{self, Error, Uri};
use std::task::{Context, Poll};

pub struct ProxyConnector<C> {
    inner: C,
}

impl<C> Service<Uri> for ProxyConnector<C> {
    type Response = Upgraded;

    type Error = Error;

    type Future = BoxFuture<'static, hyper::Result<Self::Response>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<hyper::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Uri) -> Self::Future {
        todo!()
    }
}
