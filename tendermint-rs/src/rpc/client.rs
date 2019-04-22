//! Tendermint RPC client

#![allow(unused_imports)]

use crate::{
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

    /// `/block`: get block at a given height.
    // TODO(tarcieri): support for getting latest block
    pub fn block<H>(&self, height: H) -> Result<block::Response, Error>
    where
        H: Into<Height>,
    {
        self.perform(block::Request::new(height.into()))
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

    /// `/commit`: get block commit at a given height.
    // TODO(tarcieri): support for getting latest block
    pub fn commit<H>(&self, height: H) -> Result<commit::Response, Error>
    where
        H: Into<Height>,
    {
        self.perform(commit::Request::new(height.into()))
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
            .get(&format!("http://{}:{}{}", host, port, request.path()))
            .headers(headers)
            .send()
            .map_err(Error::server_error)?;

        let mut response_body = Vec::new();
        res.read_to_end(&mut response_body)
            .map_err(Error::server_error)?;

        R::Response::from_json(&response_body)
    }
}
