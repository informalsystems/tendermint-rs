//! Blocking ABCI server.

use crate::codec::blocking::Codec;
use crate::runtime::blocking::{Runtime, TcpListener, TcpStream};
use crate::server::DEFAULT_SERVER_READ_BUF_SIZE;
use crate::{Application, Result};
use log::{debug, error, info};
use tendermint::abci::request;

/// Runtime-dependent blocking ABCI server.
///
/// Blocking servers, unfortunately, cannot be terminated gracefully since they
/// block on their listener.
pub struct Server<App, Rt: Runtime> {
    app: App,
    listener: Rt::TcpListener,
    local_addr: String,
    read_buf_size: usize,
}

impl<App, Rt> Server<App, Rt>
where
    App: Application,
    Rt: Runtime,
{
    /// Start listening for incoming connections.
    pub fn listen(self) -> Result<()> {
        loop {
            match self.listener.accept() {
                Ok(r) => {
                    let (stream, addr) = r;
                    info!("Incoming connection from: {}", addr.to_string());
                    self.spawn_client_handler(stream);
                }
                Err(e) => {
                    error!("Failed to accept incoming connection: {:?}", e);
                }
            }
        }
    }

    fn spawn_client_handler(&self, stream: Rt::TcpStream) {
        let app_clone = self.app.clone();
        let read_buf_size = self.read_buf_size;
        Rt::spawn_and_forget(move || Self::handle_client(stream, app_clone, read_buf_size));
    }

    fn handle_client(stream: Rt::TcpStream, app: App, read_buf_size: usize) {
        let mut codec = Rt::ServerCodec::new(stream.into_inner(), read_buf_size);
        loop {
            let req: request::Request = match codec.next() {
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
            if let Err(e) = codec.send(res) {
                error!("Failed to write outgoing response to client: {:?}", e);
                return;
            }
        }
    }

    /// Get the local address for the server, once bound.
    pub fn local_addr(&self) -> String {
        self.local_addr.clone()
    }
}

/// Allows for construction and configuration of a blocking ABCI server.
pub struct ServerBuilder<Rt> {
    read_buf_size: usize,
    _runtime: std::marker::PhantomData<Rt>,
}

impl<Rt: Runtime> ServerBuilder<Rt> {
    /// Constructor for a server builder that allows for customization of the
    /// read buffer size.
    pub fn new(read_buf_size: usize) -> Self {
        Self {
            read_buf_size,
            _runtime: Default::default(),
        }
    }

    /// Constructor for a blocking ABCI server.
    ///
    /// Attempts to bind the specified application to the given network
    /// address.
    pub fn bind<S, App>(self, addr: S, app: App) -> Result<Server<App, Rt>>
    where
        S: AsRef<str>,
        App: Application,
    {
        let listener = Rt::TcpListener::bind(addr.as_ref())?;
        let local_addr = listener.local_addr()?;
        Ok(Server {
            app,
            listener,
            local_addr,
            read_buf_size: self.read_buf_size,
        })
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

#[cfg(feature = "runtime-std")]
/// Convenience export for when using Rust's standard library as your runtime.
pub type StdServerBuilder = ServerBuilder<crate::runtime::blocking::runtime_std::Std>;
