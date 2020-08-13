//! Tendermint RPC client

use crate::client::subscription::Subscription;
use crate::client::transport::{SubscriptionTransport, Transport};
use crate::endpoint::*;
use crate::{Error, Request};
use tendermint::abci::{self, Transaction};
use tendermint::block::Height;
use tendermint::evidence::Evidence;
use tendermint::Genesis;
use tokio::sync::mpsc;

pub mod event_listener;
pub mod subscription;
pub mod transport;

#[cfg(test)]
pub mod test_support;

/// The default number of events we buffer in a [`Subscription`] if you do not
/// specify the buffer size when creating it.
pub const DEFAULT_SUBSCRIPTION_BUF_SIZE: usize = 100;

/// The base Tendermint RPC client, which is responsible for handling most
/// requests with the exception of subscription mechanics. Once you have an RPC
/// client, you can create a [`SubscriptionClient`] using
/// [`new_subscription_client`].
#[derive(Debug)]
pub struct Client<T: Transport> {
    transport: T,
}

impl<T: Transport> Client<T> {
    /// Create a new Tendermint RPC client using the given transport layer.
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    /// `/abci_info`: get information about the ABCI application.
    pub async fn abci_info(&self) -> Result<abci_info::AbciInfo, Error> {
        Ok(self.perform(abci_info::Request).await?.response)
    }

    /// `/abci_query`: query the ABCI application
    pub async fn abci_query(
        &self,
        path: Option<abci::Path>,
        data: impl Into<Vec<u8>>,
        height: Option<Height>,
        prove: bool,
    ) -> Result<abci_query::AbciQuery, Error> {
        Ok(self
            .perform(abci_query::Request::new(path, data, height, prove))
            .await?
            .response)
    }

    /// `/block`: get block at a given height.
    pub async fn block(&self, height: impl Into<Height>) -> Result<block::Response, Error> {
        self.perform(block::Request::new(height.into())).await
    }

    /// `/block`: get the latest block.
    pub async fn latest_block(&self) -> Result<block::Response, Error> {
        self.perform(block::Request::default()).await
    }

    /// `/block_results`: get ABCI results for a block at a particular height.
    pub async fn block_results<H>(&self, height: H) -> Result<block_results::Response, Error>
    where
        H: Into<Height>,
    {
        self.perform(block_results::Request::new(height.into()))
            .await
    }

    /// `/block_results`: get ABCI results for the latest block.
    pub async fn latest_block_results(&self) -> Result<block_results::Response, Error> {
        self.perform(block_results::Request::default()).await
    }

    /// `/blockchain`: get block headers for `min` <= `height` <= `max`.
    ///
    /// Block headers are returned in descending order (highest first).
    ///
    /// Returns at most 20 items.
    pub async fn blockchain(
        &self,
        min: impl Into<Height>,
        max: impl Into<Height>,
    ) -> Result<blockchain::Response, Error> {
        // TODO(tarcieri): return errors for invalid params before making request?
        self.perform(blockchain::Request::new(min.into(), max.into()))
            .await
    }

    /// `/broadcast_tx_async`: broadcast a transaction, returning immediately.
    pub async fn broadcast_tx_async(
        &self,
        tx: Transaction,
    ) -> Result<broadcast::tx_async::Response, Error> {
        self.perform(broadcast::tx_async::Request::new(tx)).await
    }

    /// `/broadcast_tx_sync`: broadcast a transaction, returning the response
    /// from `CheckTx`.
    pub async fn broadcast_tx_sync(
        &self,
        tx: Transaction,
    ) -> Result<broadcast::tx_sync::Response, Error> {
        self.perform(broadcast::tx_sync::Request::new(tx)).await
    }

    /// `/broadcast_tx_sync`: broadcast a transaction, returning the response
    /// from `CheckTx`.
    pub async fn broadcast_tx_commit(
        &self,
        tx: Transaction,
    ) -> Result<broadcast::tx_commit::Response, Error> {
        self.perform(broadcast::tx_commit::Request::new(tx)).await
    }

    /// `/commit`: get block commit at a given height.
    pub async fn commit(&self, height: impl Into<Height>) -> Result<commit::Response, Error> {
        self.perform(commit::Request::new(height.into())).await
    }

    /// `/validators`: get validators a given height.
    pub async fn validators<H>(&self, height: H) -> Result<validators::Response, Error>
    where
        H: Into<Height>,
    {
        self.perform(validators::Request::new(height.into())).await
    }

    /// `/commit`: get the latest block commit
    pub async fn latest_commit(&self) -> Result<commit::Response, Error> {
        self.perform(commit::Request::default()).await
    }

    /// `/health`: get node health.
    ///
    /// Returns empty result (200 OK) on success, no response in case of an error.
    pub async fn health(&self) -> Result<(), Error> {
        self.perform(health::Request).await?;
        Ok(())
    }

    /// `/genesis`: get genesis file.
    pub async fn genesis(&self) -> Result<Genesis, Error> {
        Ok(self.perform(genesis::Request).await?.genesis)
    }

    /// `/net_info`: obtain information about P2P and other network connections.
    pub async fn net_info(&self) -> Result<net_info::Response, Error> {
        self.perform(net_info::Request).await
    }

    /// `/status`: get Tendermint status including node info, pubkey, latest
    /// block hash, app hash, block height and time.
    pub async fn status(&self) -> Result<status::Response, Error> {
        self.perform(status::Request).await
    }

    /// `/broadcast_evidence`: broadcast an evidence.
    pub async fn broadcast_evidence(&self, e: Evidence) -> Result<evidence::Response, Error> {
        self.perform(evidence::Request::new(e)).await
    }

    /// Perform a request against the RPC endpoint
    pub async fn perform<R>(&self, request: R) -> Result<R::Response, Error>
    where
        R: Request,
    {
        self.transport.request(request).await
    }

    /// Gracefully terminate the underlying connection (if relevant - depends
    /// on the underlying transport).
    pub async fn close(self) -> Result<(), Error> {
        self.transport.close().await
    }
}

/// A client solely dedicated to facilitating subscriptions to [`Event`]s.
///
/// [`Event`]: crate::event::Event
#[derive(Debug)]
pub struct SubscriptionClient<T: SubscriptionTransport> {
    transport: T,
}

/// Create a new subscription client derived from the given RPC client.
pub async fn new_subscription_client<T>(
    client: &Client<T>,
) -> Result<SubscriptionClient<T::SubscriptionTransport>, Error>
where
    T: Transport,
    <T as Transport>::SubscriptionTransport: SubscriptionTransport,
{
    Ok(SubscriptionClient::new(
        client.transport.subscription_transport().await?,
    ))
}

impl<T: SubscriptionTransport> SubscriptionClient<T> {
    fn new(transport: T) -> Self {
        Self { transport }
    }

    /// Subscribe to events generated by the given query.
    ///
    /// The `buf_size` parameter allows for control over how many events get
    /// buffered by the returned [`Subscription`]. The faster you can process
    /// incoming events, the smaller this buffer size needs to be.
    pub async fn subscribe_with_buf_size(
        &mut self,
        query: String,
        buf_size: usize,
    ) -> Result<Subscription, Error> {
        let (event_tx, event_rx) = mpsc::channel(buf_size);
        let id = self
            .transport
            .subscribe(subscribe::Request::new(query.clone()), event_tx)
            .await?;
        Ok(Subscription::new(id, query, event_rx))
    }

    /// Subscribe to events generated by the given query, using the
    /// [`DEFAULT_SUBSCRIPTION_BUF_SIZE`].
    pub async fn subscribe(&mut self, query: String) -> Result<Subscription, Error> {
        self.subscribe_with_buf_size(query, DEFAULT_SUBSCRIPTION_BUF_SIZE)
            .await
    }

    /// Terminate the given subscription and consume it.
    pub async fn unsubscribe(&mut self, subscription: Subscription) -> Result<(), Error> {
        self.transport
            .unsubscribe(
                unsubscribe::Request::new(subscription.query.clone()),
                subscription,
            )
            .await
    }

    /// Gracefully terminate the underlying connection (if relevant - depends
    /// on the underlying transport).
    pub async fn close(self) -> Result<(), Error> {
        self.transport.close().await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::client::test_support::matching_transport::{
        MethodMatcher, RequestMatchingTransport,
    };
    use crate::Method;

    // TODO: Read from a fixture in the crate.
    const ABCI_INFO_RESPONSE: &str = r#"{
  "jsonrpc": "2.0",
  "id": "",
  "result": {
    "response": {
      "data": "GaiaApp",
      "last_block_height": "488120",
      "last_block_app_hash": "2LnCw0fN+Zq/gs5SOuya/GRHUmtWftAqAkTUuoxl4g4="
    }
  }
}
"#;

    // TODO: Read from a fixture in the crate.
    const BLOCK_RESPONSE: &str = r#"{
  "jsonrpc": "2.0",
  "id": "",
  "result": {
    "block_id": {
      "hash": "4FFD15F274758E474898498A191EB8CA6FC6C466576255DA132908A12AC1674C",
      "parts": {
        "total": "1",
        "hash": "BBA710736635FA20CDB4F48732563869E90871D31FE9E7DE3D900CD4334D8775"
      }
    },
    "block": {
      "header": {
        "version": {
          "block": "10",
          "app": "1"
        },
        "chain_id": "cosmoshub-2",
        "height": "10",
        "time": "2020-03-15T16:57:08.151Z",
        "last_block_id": {
          "hash": "760E050B2404A4BC661635CA552FF45876BCD927C367ADF88961E389C01D32FF",
          "parts": {
            "total": "1",
            "hash": "485070D01F9543827B3F9BAF11BDCFFBFD2BDED0B63D7192FA55649B94A1D5DE"
          }
        },
        "last_commit_hash": "594F029060D5FAE6DDF82C7DC4612055EC7F941DFED34D43B2754008DC3BBC77",
        "data_hash": "",
        "validators_hash": "3C0A744897A1E0DBF1DEDE1AF339D65EDDCF10E6338504368B20C508D6D578DC",
        "next_validators_hash": "3C0A744897A1E0DBF1DEDE1AF339D65EDDCF10E6338504368B20C508D6D578DC",
        "consensus_hash": "048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F",
        "app_hash": "0000000000000000",
        "last_results_hash": "",
        "evidence_hash": "",
        "proposer_address": "12CC3970B3AE9F19A4B1D98BE1799F2CB923E0A3"
      },
      "data": {
        "txs": null
      },
      "evidence": {
        "evidence": null
      },
      "last_commit": {
        "height": "9",
        "round": "0",
        "block_id": {
          "hash": "760E050B2404A4BC661635CA552FF45876BCD927C367ADF88961E389C01D32FF",
          "parts": {
            "total": "1",
            "hash": "485070D01F9543827B3F9BAF11BDCFFBFD2BDED0B63D7192FA55649B94A1D5DE"
          }
        },
        "signatures": [
          {
            "block_id_flag": 2,
            "validator_address": "12CC3970B3AE9F19A4B1D98BE1799F2CB923E0A3",
            "timestamp": "2020-03-15T16:57:08.151Z",
            "signature": "GRBX/UNaf19vs5byJfAuXk2FQ05soOHmaMFCbrNBhHdNZtFKHp6J9eFwZrrG+YCxKMdqPn2tQWAes6X8kpd1DA=="
          }
        ]
      }
    }
  }
}"#;

    #[tokio::test]
    async fn mocked_client_transport() {
        let transport = RequestMatchingTransport::new(MethodMatcher::new(
            Method::AbciInfo,
            Ok(ABCI_INFO_RESPONSE.into()),
        ))
        .push(MethodMatcher::new(Method::Block, Ok(BLOCK_RESPONSE.into())));
        let client = Client::new(transport);

        let abci_info = client.abci_info().await.unwrap();
        assert_eq!("GaiaApp".to_string(), abci_info.data);

        // supplied height is irrelevant when using MethodMatcher
        let block = client.block(Height::from(1234)).await.unwrap().block;
        assert_eq!(Height::from(10), block.header.height);
    }
}
