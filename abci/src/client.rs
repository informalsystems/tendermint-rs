//! Blocking ABCI client.

use crate::codec::ClientCodec;
use crate::Error;
use std::net::{TcpStream, ToSocketAddrs};
use tendermint_proto::abci::{
    request, response, RequestApplySnapshotChunk, RequestBeginBlock, RequestCheckTx, RequestCommit,
    RequestDeliverTx, RequestEndBlock, RequestFlush, RequestInfo, RequestInitChain,
    RequestListSnapshots, RequestLoadSnapshotChunk, RequestOfferSnapshot, RequestQuery,
    ResponseApplySnapshotChunk, ResponseBeginBlock, ResponseCheckTx, ResponseCommit,
    ResponseDeliverTx, ResponseEndBlock, ResponseFlush, ResponseInfo, ResponseInitChain,
    ResponseListSnapshots, ResponseLoadSnapshotChunk, ResponseOfferSnapshot, ResponseQuery,
};
use tendermint_proto::abci::{Request, RequestEcho, ResponseEcho};

/// The size of the read buffer for the client in its receiving of responses
/// from the server.
pub const DEFAULT_CLIENT_READ_BUF_SIZE: usize = 1024;

/// Builder for a blocking ABCI client.
pub struct ClientBuilder {
    read_buf_size: usize,
}

impl ClientBuilder {
    /// Builder constructor.
    pub fn new(read_buf_size: usize) -> Self {
        Self { read_buf_size }
    }

    /// Client constructor that attempts to connect to the given network
    /// address.
    pub fn connect<A: ToSocketAddrs>(self, addr: A) -> Result<Client, Error> {
        let stream = TcpStream::connect(addr).map_err(Error::io)?;
        Ok(Client {
            codec: ClientCodec::new(stream, self.read_buf_size),
        })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            read_buf_size: DEFAULT_CLIENT_READ_BUF_SIZE,
        }
    }
}

/// Blocking ABCI client.
pub struct Client {
    codec: ClientCodec<TcpStream>,
}

macro_rules! perform {
    ($self:expr, $type:ident, $req:expr) => {
        match $self.perform(request::Value::$type($req))? {
            response::Value::$type(r) => Ok(r),
            r => {
                Err(Error::unexpected_server_response_type(stringify!($type).to_string(), r).into())
            }
        }
    };
}

impl Client {
    /// Ask the ABCI server to echo back a message.
    pub fn echo(&mut self, req: RequestEcho) -> Result<ResponseEcho, Error> {
        perform!(self, Echo, req)
    }

    /// Request information about the ABCI application.
    pub fn info(&mut self, req: RequestInfo) -> Result<ResponseInfo, Error> {
        perform!(self, Info, req)
    }

    /// To be called once upon genesis.
    pub fn init_chain(&mut self, req: RequestInitChain) -> Result<ResponseInitChain, Error> {
        perform!(self, InitChain, req)
    }

    /// Query the application for data at the current or past height.
    pub fn query(&mut self, req: RequestQuery) -> Result<ResponseQuery, Error> {
        perform!(self, Query, req)
    }

    /// Check the given transaction before putting it into the local mempool.
    pub fn check_tx(&mut self, req: RequestCheckTx) -> Result<ResponseCheckTx, Error> {
        perform!(self, CheckTx, req)
    }

    /// Signal the beginning of a new block, prior to any `DeliverTx` calls.
    pub fn begin_block(&mut self, req: RequestBeginBlock) -> Result<ResponseBeginBlock, Error> {
        perform!(self, BeginBlock, req)
    }

    /// Apply a transaction to the application's state.
    pub fn deliver_tx(&mut self, req: RequestDeliverTx) -> Result<ResponseDeliverTx, Error> {
        perform!(self, DeliverTx, req)
    }

    /// Signal the end of a block.
    pub fn end_block(&mut self, req: RequestEndBlock) -> Result<ResponseEndBlock, Error> {
        perform!(self, EndBlock, req)
    }

    pub fn flush(&mut self) -> Result<ResponseFlush, Error> {
        perform!(self, Flush, RequestFlush {})
    }

    /// Commit the current state at the current height.
    pub fn commit(&mut self) -> Result<ResponseCommit, Error> {
        perform!(self, Commit, RequestCommit {})
    }

    /// Used during state sync to discover available snapshots on peers.
    pub fn list_snapshots(&mut self) -> Result<ResponseListSnapshots, Error> {
        perform!(self, ListSnapshots, RequestListSnapshots {})
    }

    /// Called when bootstrapping the node using state sync.
    pub fn offer_snapshot(
        &mut self,
        req: RequestOfferSnapshot,
    ) -> Result<ResponseOfferSnapshot, Error> {
        perform!(self, OfferSnapshot, req)
    }

    /// Used during state sync to retrieve chunks of snapshots from peers.
    pub fn load_snapshot_chunk(
        &mut self,
        req: RequestLoadSnapshotChunk,
    ) -> Result<ResponseLoadSnapshotChunk, Error> {
        perform!(self, LoadSnapshotChunk, req)
    }

    /// Apply the given snapshot chunk to the application's state.
    pub fn apply_snapshot_chunk(
        &mut self,
        req: RequestApplySnapshotChunk,
    ) -> Result<ResponseApplySnapshotChunk, Error> {
        perform!(self, ApplySnapshotChunk, req)
    }

    fn perform(&mut self, req: request::Value) -> Result<response::Value, Error> {
        self.codec.send(Request { value: Some(req) })?;
        let res = self
            .codec
            .next()
            .ok_or_else(Error::server_connection_terminated)??;
        res.value.ok_or_else(Error::malformed_server_response)
    }
}
