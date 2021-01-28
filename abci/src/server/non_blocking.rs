//! Non-blocking ABCI server.

use crate::codec::non_blocking::Codec;
use crate::runtime::non_blocking::{ChannelNotify, Receiver, Runtime, TcpListener, TcpStream};
use crate::server::DEFAULT_SERVER_READ_BUF_SIZE;
use crate::{Application, Result};
use futures::{SinkExt, StreamExt};
use log::{debug, error, info};
use tendermint::abci::request;

/// Non-blocking ABCI server for a specific application and runtime.
pub struct Server<App, Rt: Runtime> {
    app: App,
    listener: Rt::TcpListener,
    local_addr: String,
    term_rx: <Rt::ChannelNotify as ChannelNotify>::Receiver,
    read_buf_size: usize,
}

impl<App, Rt> Server<App, Rt>
where
    App: Application,
    Rt: Runtime,
{
    /// Start listening for incoming connections.
    pub async fn listen(mut self) -> Result<()> {
        use futures::FutureExt;

        loop {
            futures::select! {
                result = self.listener.accept().fuse() => match result {
                    Ok(r) =>  {
                        let (stream, addr) = r;
                        info!("Incoming connection from: {}", addr.to_string());
                        self.spawn_client_handler(stream).await;
                    },
                    Err(e) => {
                        error!("Failed to accept incoming connection: {:?}", e);
                    }
                },
                _ = self.term_rx.recv().fuse() => {
                    info!("Server terminated");
                    return Ok(())
                }
            }
        }
    }

    async fn spawn_client_handler(&self, stream: Rt::TcpStream) {
        Rt::spawn_and_forget(Self::handle_client(
            stream,
            self.app.clone(),
            self.read_buf_size,
        ));
    }

    async fn handle_client(stream: Rt::TcpStream, app: App, read_buf_size: usize) {
        let mut codec: Rt::ServerCodec = Rt::ServerCodec::new(stream.into_inner(), read_buf_size);
        loop {
            let req: request::Request = match codec.next().await {
                Some(result) => match result {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Failed to read request from client: {}", e);
                        return;
                    }
                },
                None => {
                    info!("Client terminated connection");
                    return;
                }
            };
            debug!("Got incoming request from client: {:?}", req);
            let res = app.handle(req);
            debug!("Sending outgoing response: {:?}", res);
            if let Err(e) = codec.send(res).await {
                error!("Failed to write outgoing response to client: {}", e);
                return;
            }
        }
    }

    /// Get the local address for the server, once bound.
    pub fn local_addr(&self) -> String {
        self.local_addr.clone()
    }
}

/// Allows for construction and configuration of a non-blocking ABCI server.
pub struct ServerBuilder<Rt> {
    read_buf_size: usize,
    _runtime: std::marker::PhantomData<Rt>,
}

impl<Rt: Runtime> ServerBuilder<Rt> {
    /// Constructor allowing for customization of server parameters.
    pub fn new(read_buf_size: usize) -> Self {
        Self {
            read_buf_size,
            _runtime: Default::default(),
        }
    }

    /// Bind our ABCI application server to the given address.
    ///
    /// On success, returns our server and the sending end of a channel we can
    /// use to terminate the server while it's listening.
    pub async fn bind<S, App>(
        self,
        addr: S,
        app: App,
    ) -> Result<(
        Server<App, Rt>,
        <Rt::ChannelNotify as ChannelNotify>::Sender,
    )>
    where
        S: AsRef<str>,
        App: Application,
    {
        let listener = Rt::TcpListener::bind(addr.as_ref()).await?;
        let (term_tx, term_rx) = <Rt::ChannelNotify as ChannelNotify>::unbounded();
        let local_addr = listener.local_addr()?;
        Ok((
            Server {
                app,
                listener,
                local_addr,
                term_rx,
                read_buf_size: self.read_buf_size,
            },
            term_tx,
        ))
    }
}

impl<Rt: Runtime> Default for ServerBuilder<Rt> {
    fn default() -> Self {
        Self {
            read_buf_size: DEFAULT_SERVER_READ_BUF_SIZE,
            _runtime: Default::default(),
        }
    }
}

#[cfg(feature = "runtime-tokio")]
/// Convenience export for when using Tokio's runtime.
pub type TokioServerBuilder = ServerBuilder<crate::runtime::non_blocking::runtime_tokio::Tokio>;

#[cfg(feature = "runtime-async-std")]
/// Convenience export for when using `async-std`'s runtime.
pub type AsyncStdServerBuilder =
    ServerBuilder<crate::runtime::non_blocking::runtime_async_std::AsyncStd>;
