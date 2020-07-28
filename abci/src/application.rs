use async_trait::async_trait;
use tendermint_proto::abci::*;

/// Trait for initialization and for queries from the user.
#[async_trait]
pub trait Info: Send + Sync {
    /// Echo a string to test abci client/server implementation.
    async fn echo(&self, echo_request: RequestEcho) -> ResponseEcho {
        ResponseEcho {
            message: echo_request.message,
        }
    }

    /// Return information about the application state.
    ///
    /// # Crash Recovery
    ///
    /// On startup, Tendermint calls the [`info`] method to get the **latest committed state** of
    /// the app. The app **MUST** return information consistent with the last block it
    /// successfully completed [`commit`] for.
    ///
    /// If the app succesfully committed block `H` but not `H+1`, then
    /// - `last_block_height = H`
    /// - `last_block_app_hash = <hash returned by Commit for block H>`
    ///
    /// If the app failed during the [`commit`] of block `H`, then
    /// - `last_block_height = H-1`
    /// - `last_block_app_hash = <hash returned by Commit for block H-1, which is the hash in the
    ///   header of block H>`
    ///
    /// [`info`]: trait.Info.html#tymethod.info
    /// [`commit`]: trait.Consensus.html#tymethod.commit
    ///
    /// # Equivalent to
    ///
    /// ```rust,ignore
    /// async fn info(&self, info_request: RequestInfo) -> ResponseInfo
    /// ```
    async fn info(&self, info_request: RequestInfo) -> ResponseInfo;

    /// Set non-consensus critical application specific options.
    ///
    /// # Equivalent to
    ///
    /// ```rust,ignore
    /// async fn set_option(&self, set_option_request: RequestSetOption) -> ResponseSetOption
    /// ```
    async fn set_option(&self, _set_option_request: RequestSetOption) -> ResponseSetOption {
        Default::default()
    }

    /// Query for data from the application at current or past height.
    ///
    /// # Equivalent to
    ///
    /// ```rust,ignore
    /// async fn query(&self, query_request: RequestQuery) -> ResponseQuery
    /// ```
    async fn query(&self, _query_request: RequestQuery) -> ResponseQuery {
        Default::default()
    }
}

/// Trait for managing consensus of blockchain.
///
/// # Details
///
/// [_Consensus_] should maintain a `consensus_state` - the working state for block execution. It
/// should be updated by the calls to [`begin_block`], [`deliver_tx`], and [`end_block`] during
/// block execution and committed to disk as the **latest committed state** during [`commit`].
///
/// Updates made to the `consensus_state` by each method call must be readable by each subsequent
/// method - ie. the updates are linearizable.
///
/// [_Consensus_]: trait.Consensus.html#details
/// [`begin_block`]: trait.Consensus.html#tymethod.begin_block
/// [`deliver_tx`]: trait.Consensus.html#tymethod.deliver_tx
/// [`end_block`]: trait.Consensus.html#tymethod.end_block
/// [`commit`]: trait.Consensus.html#tymethod.commit
#[async_trait]
pub trait Consensus: Send + Sync {
    /// Called once upon genesis. Usually used to establish initial (genesis) state.
    ///
    /// # Equivalent to
    ///
    /// ```rust,ignore
    /// async fn init_chain(&self, init_chain_request: RequestInitChain) -> ResponseInitChain
    /// ```
    async fn init_chain(&self, init_chain_request: RequestInitChain) -> ResponseInitChain;

    /// Signals the beginning of a new block. Called prior to any
    /// [`deliver_tx`](trait.Consensus.html#tymethod.deliver_tx)s.
    ///
    /// # Equivalent to
    ///
    /// ```rust,ignore
    /// async fn begin_block(&self, begin_block_request: RequestBeginBlock) -> ResponseBeginBlock
    /// ```
    async fn begin_block(&self, begin_block_request: RequestBeginBlock) -> ResponseBeginBlock;

    /// Execute the transaction in full. The workhorse of the application.
    ///
    /// # Equivalent to
    ///
    /// ```rust,ignore
    /// async fn deliver_tx(&self, deliver_tx_request: RequestDeliverTx) -> ResponseDeliverTx
    /// ```
    async fn deliver_tx(&self, deliver_tx_request: RequestDeliverTx) -> ResponseDeliverTx;

    /// Signals the end of a block. Called after all transactions, prior to each
    /// [`commit`](trait.Commit.html#tymethod.commit).
    ///
    /// # Equivalent to
    ///
    /// ```rust,ignore
    /// async fn end_block(&self, end_block_request: RequestEndBlock) -> ResponseEndBlock
    /// ```
    async fn end_block(&self, end_block_request: RequestEndBlock) -> ResponseEndBlock;

    /// Persist the application state.
    ///
    /// # Details
    ///
    /// Application state should only be persisted to disk during [`commit`].
    ///
    /// Before [`commit`] is called, Tendermint locks and flushes the mempool so that no new
    /// messages will be received on the mempool connection. This provides an opportunity to
    /// safely update all three states ([_Consensus_], [_Mempool_] and [_Info_]) to the **latest
    /// committed state** at once.
    ///
    /// When [`commit`] completes, it unlocks the mempool.
    ///
    /// # Warning
    ///
    /// If the ABCI application logic processing the [`commit`] message sends a `/broadcast_tx_sync`
    /// or `/broadcast_tx_commit` and waits for the response before proceeding, it will
    /// deadlock. Executing those `broadcast_tx` calls involves acquiring a lock that is held
    /// during the [`commit`] call, so it's not possible. If you make the call to the
    /// `broadcast_tx` endpoints concurrently, that's no problem, it just can't be part of the
    /// sequential logic of the [`commit`] function.
    ///
    /// [`commit`]: trait.Commit.html#tymethod.commit
    /// [_Consensus_]: trait.Consensus.html#details
    /// [_Mempool_]: trait.Mempool.html#details
    /// [_Info_]: trait.Info.html
    ///
    /// # Equivalent to
    ///
    /// ```rust,ignore
    /// async fn commit(&self, commit_request: RequestCommit) -> ResponseCommit
    /// ```
    async fn commit(&self, commit_request: RequestCommit) -> ResponseCommit;

    /// Signals that messages queued on the client should be flushed to the server.
    ///
    /// # Equivalent to
    ///
    /// ```rust,ignore
    /// async fn flush(&self, flush_request: RequestFlush) -> ResponseFlush
    /// ```
    async fn flush(&self, _flush_request: RequestFlush) -> ResponseFlush {
        Default::default()
    }
}

/// Trait for managing tendermint's mempool.
///
/// # Details
///
/// [_Mempool_] should maintain a `mempool_state` to sequentially process pending transactions in
/// the mempool that have not yet been committed. It should be initialized to the latest committed
/// state at the end of every [`commit`].
///
/// The `mempool_state` may be updated concurrently with the `consensus_state`, as messages may be
/// sent concurrently on [_Consensus_] and [_Mempool_] connections. However, before calling
/// [`commit`], Tendermint will lock and flush the mempool connection, ensuring that all existing
/// [`check_tx`] are responded to and no new ones can begin.
///
/// After [`commit`], [`check_tx`] is run again on all transactions that remain in the node's local
/// mempool after filtering those included in the block. To prevent the mempool from rechecking all
/// transactions every time a block is committed, set the configuration option
/// `mempool.recheck=false`.
///
/// Finally, the mempool will unlock and new transactions can be processed through [`check_tx`]
/// again.
///
/// Note that [`check_tx`] doesn't have to check everything that affects transaction validity; the
/// expensive things can be skipped. In fact, [`check_tx`] doesn't have to check anything; it might
/// say that any transaction is a valid transaction. Unlike [`deliver_tx`], [`check_tx`] is just
/// there as a sort of weak filter to keep invalid transactions out of the blockchain. It's weak,
/// because a Byzantine node doesn't care about [`check_tx`]; it can propose a block full of invalid
/// transactions if it wants.
///
/// [_Mempool_]: trait.Mempool.html#details
/// [`commit`]: trait.Consensus.html#tymethod.commit
/// [_Consensus_]: trait.Consensus.html#details
/// [`deliver_tx`]: trait.Consensus.html#tymethod.deliver_tx
/// [`check_tx`]: trait.Mempool.html#method.check_tx
#[async_trait]
pub trait Mempool: Send + Sync {
    /// Guardian of the mempool: every node runs CheckTx before letting a transaction into its local
    /// mempool. Technically optional - not involved in processing blocks
    ///
    /// # Equivalent to
    ///
    /// ```rust,ignore
    /// async fn check_tx(&self, check_tx_request: RequestCheckTx) -> ResponseCheckTx
    /// ```
    async fn check_tx(&self, check_tx_request: RequestCheckTx) -> ResponseCheckTx;
}

/// Trait for serving and restoring tendermint's state sync snapshots.
///
/// # Details
///
/// State sync allows new nodes to rapidly bootstrap by discovering, fetching, and applying state
/// machine snapshots instead of replaying historical blocks. For more details, see the state sync
/// section.
///
/// When a new node is discovering snapshots in the P2P network, existing nodes will call
/// [`list_snapshots`] on the application to retrieve any local state snapshots. The new node will
/// offer these snapshots to its local application via [`offer_snapshot`].
///
/// Once the application accepts a snapshot and begins restoring it, Tendermint will fetch snapshot
/// chunks from existing nodes via [`load_snapshot_chunk`] and apply them sequentially to the local
/// application with `apply_snapshot_chunk`. When all chunks have been applied, the application
/// `app_hash` is retrieved via an [`info`] query and compared to the blockchain's `app_hash`
/// verified via light client.
///
/// [`list_snapshots`]: trait.StateSync.html#method.list_snapshots
/// [`offer_snapshot`]: trait.StateSync.html#method.offer_snapshot
/// [`load_snapshot_chunk`]: trait.StateSync.html#method.load_snapshot_chunk
/// [`apply_snapshot_chunk`]: trait.StateSync.html#method.apply_snapshot_chunk
/// [`info`]: trait.Info.html#tymethod.info
#[async_trait]
pub trait Snapshot: Send + Sync {
    /// # Equivalent to
    ///
    /// ```rust,ignore
    /// async fn list_snapshots(&self, list_snapshots_request: RequestListSnapshots) -> ResponseListSnapshots
    /// ```
    async fn list_snapshots(
        &self,
        _list_snapshots_request: RequestListSnapshots,
    ) -> ResponseListSnapshots {
        Default::default()
    }

    /// # Equivalent to
    ///
    /// ```rust,ignore
    /// async fn offer_snapshot(&self: offer_snapshot_request: RequestOfferSnapshot) -> ResponseOfferSnapshot
    /// ```
    async fn offer_snapshot(
        &self,
        _offer_snapshot_request: RequestOfferSnapshot,
    ) -> ResponseOfferSnapshot {
        Default::default()
    }

    /// # Equivalent to
    ///
    /// ```rust,ignore
    /// async fn load_snapshot_chunk(&self, load_snapshot_chunk_request: RequestLoadSnapshotChunk) -> ResponseLoadSnapshotChunk
    /// ```
    async fn load_snapshot_chunk(
        &self,
        _load_snapshot_chunk_request: RequestLoadSnapshotChunk,
    ) -> ResponseLoadSnapshotChunk {
        Default::default()
    }

    /// # Equivalent to
    ///
    /// ```rust,ignore
    /// async fn apply_snapshot_chunk(&self: apply_snapshot_chunk_request: RequestApplySnapshotChunk) -> ResponseApplySnapshotChunk
    /// ```
    async fn apply_snapshot_chunk(
        &self,
        _apply_snapshot_chunk_request: RequestApplySnapshotChunk,
    ) -> ResponseApplySnapshotChunk {
        Default::default()
    }
}
