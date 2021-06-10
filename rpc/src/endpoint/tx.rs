//! `/tx` endpoint JSON-RPC wrapper

use crate::Method;
use serde::{Deserialize, Serialize};
use tendermint::{abci, block};
use tendermint_proto::types::TxProof;

/// Request for finding a transaction by its hash.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    pub hash: String,
    pub prove: bool,
}

impl Request {
    /// Constructor.
    pub fn new(hash: abci::transaction::Hash, prove: bool) -> Self {
        Self {
            hash: format!("0x{}", &hash),
            prove,
        }
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
    pub hash: abci::transaction::Hash,
    pub height: block::Height,
    pub index: u32,
    pub tx_result: abci::DeliverTx,
    pub tx: abci::Transaction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<TxProof>,
}

impl crate::Response for Response {}
