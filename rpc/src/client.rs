//! Tendermint RPC client.

mod subscription;
pub use subscription::{Subscription, SubscriptionClient};
pub mod sync;

mod transport;
pub use transport::mock::{MockClient, MockRequestMatcher, MockRequestMethodMatcher};

#[cfg(feature = "http-client")]
pub use transport::http::HttpClient;
#[cfg(feature = "websocket-client")]
pub use transport::websocket::{WebSocketClient, WebSocketClientDriver};

use crate::endpoint::*;
use crate::query::Query;
use crate::{Order, Result, SimpleRequest};
use async_trait::async_trait;
use tendermint::abci::{self, Transaction};
use tendermint::block::Height;
use tendermint::evidence::Evidence;
use tendermint::Genesis;

/// Provides lightweight access to the Tendermint RPC. It gives access to all
/// endpoints with the exception of the event subscription-related ones.
///
/// To access event subscription capabilities, use a client that implements the
/// [`SubscriptionClient`] trait.
///
/// [`SubscriptionClient`]: trait.SubscriptionClient.html
#[async_trait]
pub trait Client {
    /// `/abci_info`: get information about the ABCI application.
    async fn abci_info(&self) -> Result<abci_info::AbciInfo> {
        Ok(self.perform(abci_info::Request).await?.response)
    }

    /// `/abci_query`: query the ABCI application
    async fn abci_query<V>(
        &self,
        path: Option<abci::Path>,
        data: V,
        height: Option<Height>,
        prove: bool,
    ) -> Result<abci_query::AbciQuery>
    where
        V: Into<Vec<u8>> + Send,
    {
        Ok(self
            .perform(abci_query::Request::new(path, data, height, prove))
            .await?
            .response)
    }

    /// `/block`: get block at a given height.
    async fn block<H>(&self, height: H) -> Result<block::Response>
    where
        H: Into<Height> + Send,
    {
        self.perform(block::Request::new(height.into())).await
    }

    /// `/block`: get the latest block.
    async fn latest_block(&self) -> Result<block::Response> {
        self.perform(block::Request::default()).await
    }

    /// `/block_results`: get ABCI results for a block at a particular height.
    async fn block_results<H>(&self, height: H) -> Result<block_results::Response>
    where
        H: Into<Height> + Send,
    {
        self.perform(block_results::Request::new(height.into()))
            .await
    }

    /// `/block_results`: get ABCI results for the latest block.
    async fn latest_block_results(&self) -> Result<block_results::Response> {
        self.perform(block_results::Request::default()).await
    }

    /// `/blockchain`: get block headers for `min` <= `height` <= `max`.
    ///
    /// Block headers are returned in descending order (highest first).
    ///
    /// Returns at most 20 items.
    async fn blockchain<H>(&self, min: H, max: H) -> Result<blockchain::Response>
    where
        H: Into<Height> + Send,
    {
        // TODO(tarcieri): return errors for invalid params before making request?
        self.perform(blockchain::Request::new(min.into(), max.into()))
            .await
    }

    /// `/broadcast_tx_async`: broadcast a transaction, returning immediately.
    async fn broadcast_tx_async(&self, tx: Transaction) -> Result<broadcast::tx_async::Response> {
        self.perform(broadcast::tx_async::Request::new(tx)).await
    }

    /// `/broadcast_tx_sync`: broadcast a transaction, returning the response
    /// from `CheckTx`.
    async fn broadcast_tx_sync(&self, tx: Transaction) -> Result<broadcast::tx_sync::Response> {
        self.perform(broadcast::tx_sync::Request::new(tx)).await
    }

    /// `/broadcast_tx_sync`: broadcast a transaction, returning the response
    /// from `CheckTx`.
    async fn broadcast_tx_commit(&self, tx: Transaction) -> Result<broadcast::tx_commit::Response> {
        self.perform(broadcast::tx_commit::Request::new(tx)).await
    }

    /// `/commit`: get block commit at a given height.
    async fn commit<H>(&self, height: H) -> Result<commit::Response>
    where
        H: Into<Height> + Send,
    {
        self.perform(commit::Request::new(height.into())).await
    }

    /// `/consensus_state`: get current consensus state
    async fn consensus_state(&self) -> Result<consensus_state::Response> {
        self.perform(consensus_state::Request::new()).await
    }

    /// `/validators`: get validators a given height.
    async fn validators<H>(&self, height: H) -> Result<validators::Response>
    where
        H: Into<Height> + Send,
    {
        self.perform(validators::Request::new(height.into())).await
    }

    /// `/commit`: get the latest block commit
    async fn latest_commit(&self) -> Result<commit::Response> {
        self.perform(commit::Request::default()).await
    }

    /// `/health`: get node health.
    ///
    /// Returns empty result (200 OK) on success, no response in case of an error.
    async fn health(&self) -> Result<()> {
        self.perform(health::Request).await?;
        Ok(())
    }

    /// `/genesis`: get genesis file.
    async fn genesis(&self) -> Result<Genesis> {
        Ok(self.perform(genesis::Request).await?.genesis)
    }

    /// `/net_info`: obtain information about P2P and other network connections.
    async fn net_info(&self) -> Result<net_info::Response> {
        self.perform(net_info::Request).await
    }

    /// `/status`: get Tendermint status including node info, pubkey, latest
    /// block hash, app hash, block height and time.
    async fn status(&self) -> Result<status::Response> {
        self.perform(status::Request).await
    }

    /// `/broadcast_evidence`: broadcast an evidence.
    async fn broadcast_evidence(&self, e: Evidence) -> Result<evidence::Response> {
        self.perform(evidence::Request::new(e)).await
    }

    /// `/tx_search`: search for transactions with their results.
    async fn tx_search(
        &self,
        query: Query,
        prove: bool,
        page: u32,
        per_page: u8,
        order: Order,
    ) -> Result<tx_search::Response> {
        self.perform(tx_search::Request::new(query, prove, page, per_page, order))
            .await
    }

    /// Perform a request against the RPC endpoint
    async fn perform<R>(&self, request: R) -> Result<R::Response>
    where
        R: SimpleRequest;
}
