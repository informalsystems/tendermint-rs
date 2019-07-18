//! Tendermint RPC client

use crate::{
    abci::{self, Transaction},
    block::Height,
    net,
    rpc::{self, endpoint::*, Error, Response},
    Genesis,
};
use hyper::header;
use std::io::Read;

/// Tendermint RPC client.
///
/// Presently supports JSONRPC via HTTP.
pub struct Client {
    /// Address of the RPC server
    address: net::Address,
}

impl Client {
    /// Create a new Tendermint RPC client, connecting to the given address
    pub fn new(address: &net::Address) -> Result<Self, Error> {
        let client = Client {
            address: address.clone(),
        };
        client.health()?;
        Ok(client)
    }

    /// `/abci_info`: get information about the ABCI application.
    pub fn abci_info(&self) -> Result<abci_info::AbciInfo, Error> {
        Ok(self.perform(abci_info::Request)?.response)
    }

    /// `/abci_query`: query the ABCI application
    pub fn abci_query<D>(
        &self,
        path: Option<abci::Path>,
        data: D,
        height: Option<Height>,
        prove: bool,
    ) -> Result<abci_query::AbciQuery, Error>
    where
        D: Into<Vec<u8>>,
    {
        Ok(self
            .perform(abci_query::Request::new(path, data, height, prove))?
            .response)
    }

    /// `/block`: get block at a given height.
    pub fn block<H>(&self, height: H) -> Result<block::Response, Error>
    where
        H: Into<Height>,
    {
        self.perform(block::Request::new(height.into()))
    }

    /// `/block`: get the latest block.
    pub fn latest_block(&self) -> Result<block::Response, Error> {
        self.perform(block::Request::default())
    }

    /// `/block_results`: get ABCI results for a block at a particular height.
    pub fn block_results<H>(&self, height: H) -> Result<block_results::Response, Error>
    where
        H: Into<Height>,
    {
        self.perform(block_results::Request::new(height.into()))
    }

    /// `/block_results`: get ABCI results for the latest block.
    pub fn latest_block_results(&self) -> Result<block_results::Response, Error> {
        self.perform(block_results::Request::default())
    }

    /// `/blockchain`: get block headers for `min` <= `height` <= `max`.
    ///
    /// Block headers are returned in descending order (highest first).
    ///
    /// Returns at most 20 items.
    pub fn blockchain<H>(&self, min: H, max: H) -> Result<blockchain::Response, Error>
    where
        H: Into<Height>,
    {
        // TODO(tarcieri): return errors for invalid params before making request?
        self.perform(blockchain::Request::new(min.into(), max.into()))
    }

    /// `/broadcast_tx_async`: broadcast a transaction, returning immediately.
    pub fn broadcast_tx_async(
        &self,
        tx: Transaction,
    ) -> Result<broadcast::tx_async::Response, Error> {
        self.perform(broadcast::tx_async::Request::new(tx))
    }

    /// `/broadcast_tx_sync`: broadcast a transaction, returning the response
    /// from `CheckTx`.
    pub fn broadcast_tx_sync(
        &self,
        tx: Transaction,
    ) -> Result<broadcast::tx_sync::Response, Error> {
        self.perform(broadcast::tx_sync::Request::new(tx))
    }

    /// `/broadcast_tx_sync`: broadcast a transaction, returning the response
    /// from `CheckTx`.
    pub fn broadcast_tx_commit(
        &self,
        tx: Transaction,
    ) -> Result<broadcast::tx_commit::Response, Error> {
        self.perform(broadcast::tx_commit::Request::new(tx))
    }

    /// `/commit`: get block commit at a given height.
    pub fn commit<H>(&self, height: H) -> Result<commit::Response, Error>
    where
        H: Into<Height>,
    {
        self.perform(commit::Request::new(height.into()))
    }

    /// `/commit`: get the latest block commit
    pub fn latest_commit(&self) -> Result<commit::Response, Error> {
        self.perform(commit::Request::default())
    }

    /// `/health`: get node health.
    ///
    /// Returns empty result (200 OK) on success, no response in case of an error.
    pub fn health(&self) -> Result<(), Error> {
        self.perform(health::Request)?;
        Ok(())
    }

    /// `/genesis`: get genesis file.
    pub fn genesis(&self) -> Result<Genesis, Error> {
        Ok(self.perform(genesis::Request)?.genesis)
    }

    /// `/net_info`: obtain information about P2P and other network connections.
    pub fn net_info(&self) -> Result<net_info::Response, Error> {
        self.perform(net_info::Request)
    }

    /// `/status`: get Tendermint status including node info, pubkey, latest
    /// block hash, app hash, block height and time.
    pub fn status(&self) -> Result<status::Response, Error> {
        self.perform(status::Request)
    }

    /// Perform a request against the RPC endpoint
    pub fn perform<R>(&self, request: R) -> Result<R::Response, Error>
    where
        R: rpc::Request,
    {
        let request_body = request.into_json();

        let (host, port) = match &self.address {
            net::Address::Tcp { host, port, .. } => (host, port),
            other => Err(Error::invalid_params(&format!(
                "invalid RPC address: {:?}",
                other
            )))?,
        };

        let mut headers = hyper::header::Headers::new();

        // TODO(tarcieri): persistent connections
        headers.set(header::Connection::close());
        headers.set(header::ContentType::json());
        headers.set(header::UserAgent("tendermint.rs RPC client".to_owned()));

        let http_client = hyper::Client::new();

        let mut res = http_client
            .request(hyper::Post, &format!("http://{}:{}/", host, port))
            .headers(headers)
            .body(&request_body[..])
            .send()
            .map_err(Error::server_error)?;

        let mut response_body = Vec::new();
        res.read_to_end(&mut response_body)
            .map_err(Error::server_error)?;

        R::Response::from_json(&response_body)
    }
}
