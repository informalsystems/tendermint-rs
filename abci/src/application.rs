//! ABCI application-related types.

use async_trait::async_trait;
use tendermint_proto::abci::*;

/// ABCI application interface.
///
/// All ABCI applications need to implement this interface.
///
/// The application architectural model assumed here is that the object
/// implementing this trait can be cloned to be sent to multiple connection
/// handlers. This is because, when connecting over a socket, Tendermint opens
/// 4 distinct connections to the application:
///
/// 1. The info/query connection.
/// 2. The mempool connection, to validate transactions.
/// 3. The consensus connection, to allow the application to influence the
///    progression of the Tendermint consensus mechanism.
/// 4. The state sync connection.
#[async_trait]
pub trait Application: Clone + Send {
    // Info/query connection

    /// Return information about the application.
    async fn info(&self, request: RequestInfo) -> ResponseInfo;
    /// Query the application for state.
    async fn query(&self, request: RequestQuery) -> ResponseQuery;

    // Mempool connection

    /// Validate a transaction for the mempool.
    async fn check_tx(&self, request: RequestCheckTx) -> ResponseCheckTx;

    // Consensus connection

    /// Initialize the blockchain with validators and/or other information.
    async fn init_chain(&self, request: RequestInitChain) -> ResponseInitChain;
    /// Signals the beginning of a block.
    async fn begin_block(&self, request: RequestBeginBlock) -> ResponseBeginBlock;
    /// Deliver a transaction for full processing.
    async fn deliver_tx(&self, request: RequestDeliverTx) -> ResponseDeliverTx;
    /// Signals the end of a block, returning changes to the validator set.
    async fn end_block(&self, request: RequestEndBlock) -> ResponseEndBlock;
    /// Commit the state and return the application state hash.
    async fn commit(&self) -> ResponseCommit;

    // State sync connection

    /// List available snapshots.
    async fn list_snapshots(&self, request: RequestListSnapshots) -> ResponseListSnapshots;
    /// Offer a snapshot to the application.
    async fn offer_snapshot(&self, request: RequestOfferSnapshot) -> ResponseOfferSnapshot;
    /// Load a snapshot chunk.
    async fn load_snapshot_chunk(
        &self,
        request: RequestLoadSnapshotChunk,
    ) -> ResponseLoadSnapshotChunk;
    /// Apply a snapshot chunk.
    async fn apply_snapshot_chunk(
        &self,
        request: RequestApplySnapshotChunk,
    ) -> ResponseApplySnapshotChunk;
}
