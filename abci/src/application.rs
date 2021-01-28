//! ABCI application interface.

#[cfg(feature = "echo-app")]
pub mod echo;
#[cfg(feature = "kvstore-app")]
pub mod kvstore;

use tendermint_proto::abci::request::Value;
use tendermint_proto::abci::{
    response, Request, RequestApplySnapshotChunk, RequestBeginBlock, RequestCheckTx,
    RequestDeliverTx, RequestEcho, RequestEndBlock, RequestInfo, RequestInitChain,
    RequestLoadSnapshotChunk, RequestOfferSnapshot, RequestQuery, RequestSetOption, Response,
    ResponseApplySnapshotChunk, ResponseBeginBlock, ResponseCheckTx, ResponseCommit,
    ResponseDeliverTx, ResponseEcho, ResponseEndBlock, ResponseFlush, ResponseInfo,
    ResponseInitChain, ResponseListSnapshots, ResponseLoadSnapshotChunk, ResponseOfferSnapshot,
    ResponseQuery, ResponseSetOption,
};

/// An ABCI application.
pub trait Application: Send + Clone + 'static {
    /// Echo back the same message as provided in the request.
    fn echo(&self, request: RequestEcho) -> ResponseEcho {
        ResponseEcho {
            message: request.message,
        }
    }

    /// Provide information about the ABCI application.
    fn info(&self, _request: RequestInfo) -> ResponseInfo {
        ResponseInfo {
            data: "".to_string(),
            version: "".to_string(),
            app_version: 0,
            last_block_height: 0,
            last_block_app_hash: vec![],
        }
    }

    /// Called once upon genesis.
    fn init_chain(&self, _request: RequestInitChain) -> ResponseInitChain {
        ResponseInitChain {
            consensus_params: None,
            validators: vec![],
            app_hash: vec![],
        }
    }

    /// Query the application for data at the current or past height.
    fn query(&self, _request: RequestQuery) -> ResponseQuery {
        ResponseQuery {
            code: 0,
            log: "".to_string(),
            info: "".to_string(),
            index: 0,
            key: vec![],
            value: vec![],
            proof_ops: None,
            height: 0,
            codespace: "".to_string(),
        }
    }

    /// Check the given transaction before putting it into the local mempool.
    fn check_tx(&self, _request: RequestCheckTx) -> ResponseCheckTx {
        ResponseCheckTx {
            code: 0,
            data: vec![],
            log: "".to_string(),
            info: "".to_string(),
            gas_wanted: 0,
            gas_used: 0,
            events: vec![],
            codespace: "".to_string(),
        }
    }

    /// Signals the beginning of a new block, prior to any `DeliverTx` calls.
    fn begin_block(&self, _request: RequestBeginBlock) -> ResponseBeginBlock {
        ResponseBeginBlock { events: vec![] }
    }

    /// Apply a transaction to the application's state.
    fn deliver_tx(&self, _request: RequestDeliverTx) -> ResponseDeliverTx {
        ResponseDeliverTx {
            code: 0,
            data: vec![],
            log: "".to_string(),
            info: "".to_string(),
            gas_wanted: 0,
            gas_used: 0,
            events: vec![],
            codespace: "".to_string(),
        }
    }

    /// Signals the end of a block.
    fn end_block(&self, _request: RequestEndBlock) -> ResponseEndBlock {
        ResponseEndBlock {
            validator_updates: vec![],
            consensus_param_updates: None,
            events: vec![],
        }
    }

    /// Signals that messages queued on the client should be flushed to the server.
    fn flush(&self) -> ResponseFlush {
        ResponseFlush {}
    }

    /// Commit the current state at the current height.
    fn commit(&self) -> ResponseCommit {
        ResponseCommit {
            data: vec![],
            retain_height: 0,
        }
    }

    /// Allows the Tendermint node to request that the application set an
    /// option to a particular value.
    fn set_option(&self, _request: RequestSetOption) -> ResponseSetOption {
        ResponseSetOption {
            code: 0,
            log: "".to_string(),
            info: "".to_string(),
        }
    }

    /// Used during state sync to discover available snapshots on peers.
    fn list_snapshots(&self) -> ResponseListSnapshots {
        ResponseListSnapshots { snapshots: vec![] }
    }

    /// Called when bootstrapping the node using state sync.
    fn offer_snapshot(&self, _request: RequestOfferSnapshot) -> ResponseOfferSnapshot {
        ResponseOfferSnapshot { result: 0 }
    }

    /// Used during state sync to retrieve chunks of snapshots from peers.
    fn load_snapshot_chunk(&self, _request: RequestLoadSnapshotChunk) -> ResponseLoadSnapshotChunk {
        ResponseLoadSnapshotChunk { chunk: vec![] }
    }

    /// Apply the given snapshot chunk to the application's state.
    fn apply_snapshot_chunk(
        &self,
        _request: RequestApplySnapshotChunk,
    ) -> ResponseApplySnapshotChunk {
        ResponseApplySnapshotChunk {
            result: 0,
            refetch_chunks: vec![],
            reject_senders: vec![],
        }
    }

    /// Executes the relevant application method based on the type of the
    /// request, and produces the corresponding response.
    fn handle(&self, request: Request) -> Response {
        Response {
            value: Some(match request.value.unwrap() {
                Value::Echo(req) => response::Value::Echo(self.echo(req)),
                Value::Flush(_) => response::Value::Flush(self.flush()),
                Value::Info(req) => response::Value::Info(self.info(req)),
                Value::SetOption(req) => response::Value::SetOption(self.set_option(req)),
                Value::InitChain(req) => response::Value::InitChain(self.init_chain(req)),
                Value::Query(req) => response::Value::Query(self.query(req)),
                Value::BeginBlock(req) => response::Value::BeginBlock(self.begin_block(req)),
                Value::CheckTx(req) => response::Value::CheckTx(self.check_tx(req)),
                Value::DeliverTx(req) => response::Value::DeliverTx(self.deliver_tx(req)),
                Value::EndBlock(req) => response::Value::EndBlock(self.end_block(req)),
                Value::Commit(_) => response::Value::Commit(self.commit()),
                Value::ListSnapshots(_) => response::Value::ListSnapshots(self.list_snapshots()),
                Value::OfferSnapshot(req) => {
                    response::Value::OfferSnapshot(self.offer_snapshot(req))
                }
                Value::LoadSnapshotChunk(req) => {
                    response::Value::LoadSnapshotChunk(self.load_snapshot_chunk(req))
                }
                Value::ApplySnapshotChunk(req) => {
                    response::Value::ApplySnapshotChunk(self.apply_snapshot_chunk(req))
                }
            }),
        }
    }
}
