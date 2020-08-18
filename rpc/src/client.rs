//! Tendermint RPC client.

mod subscription;
pub use subscription::{
    EventRx, EventTx, PendingResultTx, Subscription, SubscriptionId, SubscriptionRouter,
};

mod transport;
#[cfg(feature = "http_ws")]
pub use transport::{HttpClient, HttpWebSocketClient};

use crate::endpoint::*;
use crate::{Request, Result};
use async_trait::async_trait;
use tendermint::abci::{self, Transaction};
use tendermint::block::Height;
use tendermint::evidence::Evidence;
use tendermint::Genesis;

/// The default number of events we buffer in a [`Subscription`] if you do not
/// specify the buffer size when creating it.
///
/// [`Subscription`]: struct.Subscription.html
///
pub const DEFAULT_SUBSCRIPTION_BUF_SIZE: usize = 100;

/// Provides lightweight access to the Tendermint RPC. It gives access to all
/// endpoints with the exception of the event subscription-related ones.
///
/// To access event subscription capabilities, rather use a client that
/// implements the [`FullClient`] trait.
///
/// [`FullClient`]: trait.FullClient.html
///
#[async_trait]
pub trait MinimalClient {
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

    /// Perform a request against the RPC endpoint
    async fn perform<R>(&self, request: R) -> Result<R::Response>
    where
        R: Request;

    /// Gracefully terminate the underlying connection (if relevant - depends
    /// on the underlying transport).
    async fn close(self) -> Result<()>;
}

/// A client that augments a [`MinimalClient`] functionality with subscription
/// capabilities.
///
/// [`MinimalClient`]: trait.MinimalClient.html
///
#[async_trait]
pub trait FullClient: MinimalClient {
    /// `/subscribe`: subscribe to receive events produced by the given query.
    ///
    /// Allows for specification of the `buf_size` parameter, which determines
    /// how many events can be buffered in the resulting [`Subscription`]. The
    /// size of this buffer must be tuned according to how quickly your
    /// application can process the incoming events from this particular query.
    /// The slower your application processes events, the larger this buffer
    /// needs to be.
    ///
    /// [`Subscription`]: struct.Subscription.html
    ///
    async fn subscribe_with_buf_size(
        &mut self,
        query: String,
        buf_size: usize,
    ) -> Result<Subscription>;

    /// `/subscribe`: subscribe to receive events produced by the given query.
    ///
    /// Uses [`DEFAULT_SUBSCRIPTION_BUF_SIZE`] as the buffer size for the
    /// returned [`Subscription`].
    ///
    /// [`DEFAULT_SUBSCRIPTION_BUF_SIZE`]: constant.DEFAULT_SUBSCRIPTION_BUF_SIZE.html
    /// [`Subscription`]: struct.Subscription.html
    ///
    async fn subscribe(&mut self, query: String) -> Result<Subscription> {
        self.subscribe_with_buf_size(query, DEFAULT_SUBSCRIPTION_BUF_SIZE)
            .await
    }

    /// `/unsubscribe`: unsubscribe from receiving events for the given
    /// subscription.
    ///
    /// This terminates the given subscription and consumes it, since it is no
    /// longer usable.
    async fn unsubscribe(&mut self, subscription: Subscription) -> Result<()>;
}
