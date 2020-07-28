#[cfg(unix)]
use std::path::PathBuf;
use std::{io::Result, net::SocketAddr, sync::Arc};

#[cfg(all(unix, feature = "use-async-std"))]
use async_std::os::unix::net::UnixListener;
#[cfg(feature = "use-async-std")]
use async_std::{
    io::{Read, Write},
    net::TcpListener,
    prelude::*,
    sync::Mutex,
    task::spawn,
};
use tendermint_proto::abci::{
    request::Value as RequestValue, response::Value as ResponseValue, Request, Response,
};
#[cfg(all(unix, feature = "use-tokio"))]
use tokio::net::UnixListener;
#[cfg(feature = "use-tokio")]
use tokio::{
    io::{AsyncRead as Read, AsyncWrite as Write},
    net::TcpListener,
    spawn,
    stream::StreamExt,
    sync::Mutex,
};
use tracing::{debug, error, info, instrument};

use crate::{
    state::ConsensusStateValidator,
    types::{decode, encode},
    Consensus, Info, Mempool, Snapshot,
};

/// ABCI Server
pub struct Server<C, M, I, S>
where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
    S: Snapshot + 'static,
{
    /// Wrapping inner type in `Arc` so that it becomes clonable and can be shared between multiple
    /// async tasks
    pub(crate) inner: Arc<Inner<C, M, I, S>>,
}

/// Inner type that contains all the trait implementations
pub(crate) struct Inner<C, M, I, S>
where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
    S: Snapshot + 'static,
{
    consensus: C,
    mempool: M,
    info: I,
    snapshot: S,
    consensus_state: Mutex<ConsensusStateValidator>,
}

impl<C, M, I, S> Server<C, M, I, S>
where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
    S: Snapshot + 'static,
{
    /// Creates a new instance of [`Server`](struct.Server.html)
    pub fn new(consensus: C, mempool: M, info: I, snapshot: S) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(Inner {
                consensus,
                mempool,
                info,
                snapshot,
                consensus_state: Default::default(),
            }),
        })
    }

    /// Starts ABCI server
    ///
    /// # Note
    ///
    /// This is an `async` function and returns a `Future`. So, you'll need an executor to drive the
    /// `Future` returned from this function. `async-std` and `tokio` are two popular options.
    pub async fn run<T>(&self, addr: T) -> Result<()>
    where
        T: Into<Address>,
    {
        let addr = addr.into();

        match addr {
            Address::Tcp(addr) => {
                #[cfg(feature = "use-async-std")]
                let listener = TcpListener::bind(addr).await?;

                #[cfg(feature = "use-tokio")]
                let mut listener = TcpListener::bind(addr).await?;

                info!(message = "Started ABCI server at", %addr);

                let mut incoming = listener.incoming();

                while let Some(stream) = incoming.next().await {
                    let stream = stream?;
                    let peer_addr = stream.peer_addr().ok();
                    self.handle_connection(stream, peer_addr);
                }
            }
            #[cfg(unix)]
            Address::Uds(path) => {
                #[cfg(feature = "use-async-std")]
                let listener = UnixListener::bind(&path).await?;

                #[cfg(feature = "use-tokio")]
                let mut listener = UnixListener::bind(&path)?;

                info!(message = "Started ABCI server at", path = %path.display());

                let mut incoming = listener.incoming();

                while let Some(stream) = incoming.next().await {
                    let stream = stream?;
                    let peer_addr = stream.peer_addr().ok();
                    self.handle_connection(stream, peer_addr);
                }
            }
        }

        Ok(())
    }

    #[instrument(skip(self, stream))]
    pub(crate) fn handle_connection<D, P>(&self, mut stream: D, peer_addr: Option<P>)
    where
        D: Read + Write + Send + Unpin + 'static,
        P: std::fmt::Debug + Send + 'static,
    {
        info!("New peer connection");

        let inner = self.inner.clone();

        spawn(async move {
            while let Ok(request) = decode(&mut stream).await {
                match request {
                    Some(request) => {
                        let response = inner.process(request).await;

                        if let Err(err) = encode(response, &mut stream).await {
                            error!(message = "Error while writing to stream", %err, ?peer_addr);
                        }
                    }
                    None => debug!(message = "Received empty request", ?peer_addr),
                }
            }

            error!(
                message = "Error while receiving ABCI request from socket",
                ?peer_addr
            );
        });
    }
}

impl<C, M, I, S> Inner<C, M, I, S>
where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
    S: Snapshot + 'static,
{
    #[instrument(skip(self))]
    pub(crate) async fn process(&self, request: Request) -> Response {
        if request.value.is_none() {
            debug!(message = "Received a request without value", ?request);
            return Response::default();
        }

        let value = match request.value.unwrap() {
            RequestValue::Echo(request) => ResponseValue::Echo(self.info.echo(request).await),
            RequestValue::Flush(request) => {
                ResponseValue::Flush(self.consensus.flush(request).await)
            }
            RequestValue::Info(request) => {
                let info_response = self.info.info(request).await;
                self.consensus_state
                    .lock()
                    .await
                    .on_info_response(&info_response);
                ResponseValue::Info(info_response)
            }
            RequestValue::SetOption(request) => {
                ResponseValue::SetOption(self.info.set_option(request).await)
            }
            RequestValue::InitChain(request) => {
                self.consensus_state.lock().await.on_init_chain_request();
                ResponseValue::InitChain(self.consensus.init_chain(request).await)
            }
            RequestValue::Query(request) => ResponseValue::Query(self.info.query(request).await),
            RequestValue::BeginBlock(request) => {
                self.consensus_state
                    .lock()
                    .await
                    .on_begin_block_request(&request);
                ResponseValue::BeginBlock(self.consensus.begin_block(request).await)
            }
            RequestValue::CheckTx(request) => {
                ResponseValue::CheckTx(self.mempool.check_tx(request).await)
            }
            RequestValue::DeliverTx(request) => {
                self.consensus_state.lock().await.on_deliver_tx_request();
                ResponseValue::DeliverTx(self.consensus.deliver_tx(request).await)
            }
            RequestValue::EndBlock(request) => {
                self.consensus_state
                    .lock()
                    .await
                    .on_end_block_request(&request);
                ResponseValue::EndBlock(self.consensus.end_block(request).await)
            }
            RequestValue::Commit(request) => {
                let mut consensus_state = self.consensus_state.lock().await;
                consensus_state.on_commit_request();

                let response = self.consensus.commit(request).await;
                consensus_state.on_commit_response(&response);
                ResponseValue::Commit(response)
            }
            RequestValue::ListSnapshots(request) => {
                ResponseValue::ListSnapshots(self.snapshot.list_snapshots(request).await)
            }
            RequestValue::OfferSnapshot(request) => {
                ResponseValue::OfferSnapshot(self.snapshot.offer_snapshot(request).await)
            }
            RequestValue::LoadSnapshotChunk(request) => {
                ResponseValue::LoadSnapshotChunk(self.snapshot.load_snapshot_chunk(request).await)
            }
            RequestValue::ApplySnapshotChunk(request) => {
                ResponseValue::ApplySnapshotChunk(self.snapshot.apply_snapshot_chunk(request).await)
            }
        };

        let mut response = Response::default();
        response.value = Some(value);

        debug!(message = "Sending response", ?response);

        response
    }
}

/// Address of ABCI Server
#[derive(Debug)]
pub enum Address {
    /// TCP Address
    Tcp(SocketAddr),
    /// UDS Address
    #[cfg(unix)]
    #[cfg_attr(feature = "doc", doc(cfg(unix)))]
    Uds(PathBuf),
}

impl From<SocketAddr> for Address {
    fn from(addr: SocketAddr) -> Self {
        Self::Tcp(addr)
    }
}

#[cfg(unix)]
impl From<PathBuf> for Address {
    fn from(path: PathBuf) -> Self {
        Self::Uds(path)
    }
}
