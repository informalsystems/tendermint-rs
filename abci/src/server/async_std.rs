//! `async-std`-based ABCI server.

use crate::codec::{TspDecoder, TspEncoder};
use crate::{Application, Result};
use async_channel::{bounded, Receiver, Sender};
use async_std::net::{TcpListener, TcpStream, ToSocketAddrs};
use bytes::BytesMut;
use futures::{select, AsyncReadExt, AsyncWriteExt, FutureExt};
use log::info;
use tendermint::abci::request::Request;
use tendermint::abci::response::Response;

const SERVER_READ_BUF_SIZE: usize = 4096;

/// `async-std`-based ABCI server for a specific ABCI application.
///
/// Listens for incoming TCP connections.
pub struct AsyncStdServer<A> {
    app: A,
    listener: TcpListener,
    local_addr: String,
    term_rx: Receiver<()>,
}

impl<A: Application + 'static> AsyncStdServer<A> {
    /// Bind the application server to the given socket address using TCP.
    ///
    /// On success, returns the server and a channel through which the server
    /// can be asynchronously signaled to terminate.
    pub async fn bind<S>(addr: S, app: A) -> Result<(Self, Sender<()>)>
    where
        S: ToSocketAddrs,
    {
        let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;
        info!(
            "ABCI server bound to {}, listening for incoming connections",
            local_addr,
        );
        let (term_tx, term_rx) = bounded(1);
        Ok((
            Self {
                app,
                listener,
                local_addr: local_addr.to_string(),
                term_rx,
            },
            term_tx,
        ))
    }

    /// Getter for the server's local address.
    pub fn local_addr(&self) -> String {
        self.local_addr.clone()
    }

    /// Block the current task while listening for incoming connections.
    ///
    /// Each incoming connection is spawned in a separate task.
    pub async fn listen(self) -> Result<()> {
        loop {
            select! {
                result = self.listener.accept().fuse() => {
                    let (stream, addr) = result?;
                    info!("Incoming connection from {}", addr);
                    let conn_app = self.app.clone();
                    async_std::task::spawn(async move { handle_client(stream, conn_app).await });
                },
                _ = self.term_rx.recv().fuse() => {
                    // TODO(thane): Terminate client tasks
                    info!("Server terminated");
                    return Ok(())
                }
            }
        }
    }
}

// Each incoming request is processed sequentially in a single connection.
async fn handle_client<A: Application>(mut stream: TcpStream, app: A) -> Result<()> {
    let mut decoder = TspDecoder::new();
    let mut stream_buf = BytesMut::new();
    let mut read_buf = [0u8; SERVER_READ_BUF_SIZE];
    let mut write_buf = BytesMut::new();
    loop {
        let bytes_read = stream.read(&mut read_buf).await?;
        stream_buf.extend_from_slice(&read_buf[..bytes_read]);
        // Try to process as many requests as we can from the stream buffer
        'request_loop: loop {
            let request = match decoder.decode_request(&mut stream_buf)? {
                Some(req) => req,
                None => break 'request_loop,
            };
            let response = match request {
                Request::Echo(echo) => Response::Echo(app.echo(echo)),
            };
            TspEncoder::encode_response(response, &mut write_buf)?;
            stream.write(&write_buf).await?;
            write_buf.clear();
        }
    }
}
