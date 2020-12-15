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

/// Attempts to bind to the given address and immediately start serving the
/// given ABCI application in the current task.
pub async fn serve<S, A>(
    addr: S,
    app: A,
    mut term: mpsc::Receiver<()>,
    ready: mpsc::Sender<String>,
) -> Result<()>
where
    S: ToSocketAddrs,
    A: Application + 'static,
{
    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    info!(
        "ABCI server bound to {}, listening for incoming connections",
        local_addr,
    );
    ready
        .send(local_addr.to_string())
        .await
        .map_err(|e| Error::TokioChannelSend(e.to_string()))?;
    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, addr) = result?;
                info!("Incoming connection from {}", addr);
                let conn_app = app.clone();
                tokio::spawn(async move { handle_client(stream, conn_app).await });
            },
            Some(_) = term.recv() => {
                info!("Server terminated");
                return Ok(())
            }
        }
    }
}

async fn handle_client<A: Application>(stream: TcpStream, app: A) -> Result<()> {
    let codec = ServerCodec::new();
    let mut stream = Framed::new(stream, codec);
    loop {
        let request = match stream.next().await {
            Some(res) => res?,
            None => return Ok(()),
        };
        let response = match request {
            Request::Echo(echo) => Response::Echo(app.echo(echo)),
        };
        stream.send(response).await?;
    }
}

// The server decodes requests and encodes responses.
struct ServerCodec {
    decoder: TspDecoder,
}

impl ServerCodec {
    fn new() -> Self {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::application::test::EchoApp;
    use tendermint::abci::request::Echo;

    struct ClientCodec {
        decoder: TspDecoder,
    }

    impl ClientCodec {
        fn new() -> Self {
            Self {
                decoder: TspDecoder::new(),
            }
        }
    }

    impl Encoder<Request> for ClientCodec {
        type Error = Error;

        fn encode(&mut self, item: Request, mut dst: &mut BytesMut) -> Result<()> {
            TspEncoder::encode_request(item, &mut dst)
        }
    }

    impl Decoder for ClientCodec {
        type Item = Response;
        type Error = Error;

        fn decode(&mut self, mut src: &mut BytesMut) -> Result<Option<Self::Item>> {
            self.decoder.decode_response(&mut src)
        }
    }

    #[tokio::test]
    async fn echo() {
        let app = EchoApp::default();
        let (term_tx, term_rx) = mpsc::channel(1);
        let (ready_tx, mut ready_rx) = mpsc::channel(1);
        let server_handle =
            tokio::spawn(async move { serve("127.0.0.1:0", app, term_rx, ready_tx).await });

        // Wait for the server to become available
        let server_addr = ready_rx.recv().await.unwrap();

        let mut client = Framed::new(
            TcpStream::connect(server_addr).await.unwrap(),
            ClientCodec::new(),
        );
        client
            .send(Request::Echo(Echo::new("Hello ABCI :-)")))
            .await
            .unwrap();
        match client.next().await {
            Some(result) => {
                let response = result.unwrap();
                match response {
                    Response::Echo(res) => {
                        assert_eq!(res.message, "Hello ABCI :-)");
                    }
                }
            }
            None => panic!("No response from server"),
        }

        term_tx.send(()).await.unwrap();
        server_handle.await.unwrap().unwrap();
    }
}
