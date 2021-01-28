//! ABCI application server interface.

use crate::codec::ServerCodec;
use crate::{Application, Result};
use log::{error, info};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread;

/// The size of the read buffer for each incoming connection to the ABCI
/// server.
pub const DEFAULT_SERVER_READ_BUF_SIZE: usize = 1024 * 1024;

/// Allows us to configure and construct an ABCI server.
pub struct ServerBuilder {
    read_buf_size: usize,
}

impl ServerBuilder {
    /// Builder constructor.
    pub fn new(read_buf_size: usize) -> Self {
        Self { read_buf_size }
    }

    /// Constructor for an ABCI server.
    pub fn bind<Addr, App>(self, addr: Addr, app: App) -> Result<Server<App>>
    where
        Addr: ToSocketAddrs,
        App: Application,
    {
        let listener = TcpListener::bind(addr)?;
        let local_addr = listener.local_addr()?.to_string();
        Ok(Server {
            app,
            listener,
            local_addr,
            read_buf_size: self.read_buf_size,
        })
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self {
            read_buf_size: DEFAULT_SERVER_READ_BUF_SIZE,
        }
    }
}

/// A TCP-based server for serving a specific ABCI application.
pub struct Server<App> {
    app: App,
    listener: TcpListener,
    local_addr: String,
    read_buf_size: usize,
}

impl<App: Application> Server<App> {
    /// Initiate a blocking listener for incoming connections.
    pub fn listen(self) -> Result<()> {
        loop {
            let (stream, addr) = self.listener.accept()?;
            let addr = addr.to_string();
            info!("Incoming connection from: {}", addr);
            self.spawn_client_handler(stream, addr);
        }
    }

    /// Getter for this server's local address.
    pub fn local_addr(&self) -> String {
        self.local_addr.clone()
    }

    fn spawn_client_handler(&self, stream: TcpStream, addr: String) {
        let app = self.app.clone();
        let read_buf_size = self.read_buf_size;
        let _ = thread::spawn(move || Self::handle_client(stream, addr, app, read_buf_size));
    }

    fn handle_client(stream: TcpStream, addr: String, app: App, read_buf_size: usize) {
        let mut codec = ServerCodec::new(stream, read_buf_size);
        info!("Listening for incoming requests from {}", addr);
        loop {
            let request = match codec.next() {
                Some(result) => match result {
                    Ok(r) => r,
                    Err(e) => {
                        error!(
                            "Failed to read incoming request from client {}: {:?}",
                            addr, e
                        );
                        return;
                    }
                },
                None => {
                    info!("Client {} terminated stream", addr);
                    return;
                }
            };
            let response = app.handle(request);
            if let Err(e) = codec.send(response) {
                error!("Failed sending response to client {}: {:?}", addr, e);
                return;
            }
        }
    }
}
