//! Tokio-based ABCI server.

use crate::codec::{TspDecoder, TspEncoder};
use crate::{Application, Error, Result};
use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use log::info;
use tendermint::abci::request::Request;
use tendermint::abci::response::Response;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::mpsc;
use tokio_util::codec::{Decoder, Encoder, Framed};

/// Tokio-based ABCI server for a specific application.
///
/// Listens for incoming TCP connections.
pub struct TokioServer<A> {
    app: A,
    listener: TcpListener,
    local_addr: String,
    term_rx: mpsc::Receiver<()>,
}

impl<A: Application + 'static> TokioServer<A> {
    /// Bind the application server to the given socket address using TCP.
    ///
    /// On success, returns the server and a channel through which the server
    /// can be asynchronously signaled to terminate.
    pub async fn bind<S>(addr: S, app: A) -> Result<(Self, mpsc::Sender<()>)>
    where
        S: ToSocketAddrs,
    {
        let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;
        info!(
            "ABCI server bound to {}, listening for incoming connections",
            local_addr,
        );
        let (term_tx, term_rx) = mpsc::channel(1);
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
    pub async fn listen(mut self) -> Result<()> {
        loop {
            tokio::select! {
                result = self.listener.accept() => {
                    let (stream, addr) = result?;
                    info!("Incoming connection from {}", addr);
                    let conn_app = self.app.clone();
                    tokio::spawn(async move { handle_client(stream, conn_app).await });
                },
                Some(_) = self.term_rx.recv() => {
                    // TODO(thane): Terminate client tasks
                    info!("Server terminated");
                    return Ok(())
                }
            }
        }
    }
}

// Each incoming request is processed sequentially in a single connection.
async fn handle_client<A: Application>(stream: TcpStream, app: A) -> Result<()> {
    let codec = ServerCodec::default();
    let mut stream = Framed::new(stream, codec);
    loop {
        let request = match stream.next().await {
            Some(res) => res?,
            None => return Ok(()),
        };
        let response = match request {
            Request::Echo(echo) => Response::Echo(app.echo(echo)),
            Request::Info(info) => Response::Info(app.info(info)),
        };
        stream.send(response).await?;
    }
}

/// Codec for the ABCI server.
///
/// Implements [`Decode`] for [`Request`]s and [`Encode`] for [`Response`]s.
pub struct ServerCodec {
    decoder: TspDecoder,
}

impl Default for ServerCodec {
    fn default() -> Self {
        Self {
            decoder: TspDecoder::new(),
        }
    }
}

impl Encoder<Response> for ServerCodec {
    type Error = Error;

    fn encode(&mut self, item: Response, mut dst: &mut BytesMut) -> Result<()> {
        TspEncoder::encode_response(item, &mut dst)
    }
}

impl Decoder for ServerCodec {
    type Item = Request;
    type Error = Error;

    fn decode(&mut self, mut src: &mut BytesMut) -> Result<Option<Self::Item>> {
        self.decoder.decode_request(&mut src)
    }
}
