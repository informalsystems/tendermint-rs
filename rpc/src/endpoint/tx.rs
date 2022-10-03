//! `/tx` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};
use tendermint::{abci, block, Hash};
use tendermint_proto::types::TxProof;

use crate::Method;

/// Request for finding a transaction by its hash.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// The hash of the transaction we want to find.
    ///
    /// Serialized internally into a hexadecimal-encoded string before sending
    /// to the RPC server.
    #[serde(with = "crate::serializers::hash_base64")]
    pub hash: Hash,
    /// Whether or not to include the proofs of the transaction's inclusion in
    /// the block.
    pub prove: bool,
}

impl Request {
    /// Constructor.
    pub fn new(hash: Hash, prove: bool) -> Self {
        Self { hash, prove }
    }
}

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> Method {
        Method::Tx
    }
}

impl crate::SimpleRequest for Request {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// The hash of the transaction.
    ///
    /// Deserialized from a hex-encoded string (there is a discrepancy between
    /// the format used for the request and the format used for the response in
    /// the Tendermint RPC).
    pub hash: Hash,
    pub height: block::Height,
    pub index: u32,
    pub tx_result: abci::response::DeliverTx,
    pub tx: crate::abci::Transaction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<TxProof>,
}

impl crate::Response for Response {}
