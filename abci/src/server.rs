//! ABCI servers.

use crate::runtime::{ChannelNotify, Receiver, Runtime, ServerCodec, TaskSpawner, TcpListener};
use crate::{Application, Result};
use log::{debug, error, info};
use tendermint::abci::request;

/// ABCI server for a specific application and runtime.
pub struct Server<App, Rt: Runtime> {
    app: App,
    listener: Rt::TcpListener,
    local_addr: String,
    term_rx: <Rt::ChannelNotify as ChannelNotify>::Receiver,
}

#[cfg(feature = "async")]
impl<App, Rt> Server<App, Rt>
where
    App: Application,
    Rt: Runtime,
{
    /// Bind our ABCI application server to the given address.
    ///
    /// On success, returns our server and the sending end of a channel we can
    /// use to terminate the server while it's listening.
    pub async fn bind<S: AsRef<str>>(
        addr: S,
        app: App,
    ) -> Result<(Self, <Rt::ChannelNotify as ChannelNotify>::Sender)> {
        let listener = Rt::TcpListener::bind(addr.as_ref()).await?;
        let (term_tx, term_rx) = <Rt::ChannelNotify as ChannelNotify>::unbounded();
        let local_addr = listener.local_addr()?;
        Ok((
            Self {
                app,
                listener,
                local_addr,
                term_rx,
            },
            term_tx,
        ))
    }

    /// Get the local address for the server, once bound.
    pub fn local_addr(&self) -> String {
        self.local_addr.clone()
    }

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
        Rt::TaskSpawner::spawn_and_forget(Self::handle_client(stream, self.app.clone()));
    }

    async fn handle_client(stream: Rt::TcpStream, app: App) {
        use futures::{SinkExt, StreamExt};

        let mut codec = Rt::ServerCodec::from_tcp_stream(stream);
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
}
