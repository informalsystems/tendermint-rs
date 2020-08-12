//! Tendermint RPC client

use tendermint::{
    abci::{self, Transaction},
    block::Height,
    evidence::Evidence,
    net, Genesis,
};

use crate::{
    client::{
        subscription::SubscriptionManager,
        transport::{http_ws::HttpWsTransport, Transport},
    },
    endpoint::*,
    Error, Request, Response,
};

pub mod event_listener;
pub mod subscription;
pub mod transport;

#[cfg(test)]
pub mod testing;

/// Tendermint RPC client.
///
/// Presently supports JSONRPC via HTTP.
#[derive(Debug)]
pub struct Client {
    transport: Box<dyn Transport>,
}

impl Client {
    /// Create a new Tendermint RPC client, connecting to the given address.
    /// By default this uses the [`HttpWsTransport`] transport layer. This
    /// transport lazily initializes subscription mechanisms when first
    /// subscribing to events generated by a particular query.
    pub fn new(address: net::Address) -> Result<Self, Error> {
        Ok(Self {
            transport: Box::new(HttpWsTransport::new(address)?),
        })
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

    /// Creates a subscription management interface for this RPC client. This
    /// interface facilitates subscribing and unsubscribing from receiving
    /// events produced by specific RPC queries.
    pub async fn new_subscription_manager(
        &self,
        event_buf_size: usize,
    ) -> Result<SubscriptionManager, Error> {
        let conn = self.transport.new_event_connection(event_buf_size).await?;
        Ok(SubscriptionManager::new(conn, 10))
    }

    /// Perform a request against the RPC endpoint
    pub async fn perform<R>(&self, request: R) -> Result<R::Response, Error>
    where
        R: Request,
    {
        let request_body = request.into_json();
        let response_body = self.transport.request(request_body).await?;
        R::Response::from_string(response_body)
    }
}

impl From<Box<dyn Transport>> for Client {
    fn from(transport: Box<dyn Transport>) -> Self {
        Self { transport }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Method;
    use testing::matching_transport::{MethodMatcher, RequestMatchingTransport};

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
    async fn mocked_transport() {
        let mt = RequestMatchingTransport::new(MethodMatcher::new(
            Method::AbciInfo,
            Ok(ABCI_INFO_RESPONSE.into()),
        ))
        .push(MethodMatcher::new(Method::Block, Ok(BLOCK_RESPONSE.into())));

        let transport: Box<dyn Transport> = Box::new(mt);
        let client = Client::from(transport);

        let abci_info = client.abci_info().await.unwrap();
        assert_eq!("GaiaApp".to_string(), abci_info.data);

        // supplied height is irrelevant when using MethodMatcher
        let block = client.block(Height::from(1234)).await.unwrap().block;
        assert_eq!(Height::from(10), block.header.height);
    }
}
