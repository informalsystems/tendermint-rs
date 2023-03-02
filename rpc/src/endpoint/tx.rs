//! `/tx` endpoint JSON-RPC wrapper

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tendermint::{abci, block, tx, Hash};

use crate::dialect::{DeliverTx, Dialect};
use crate::{prelude::*, request::RequestMessage, serializers, Method};

/// Request for finding a transaction by its hash.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// The hash of the transaction we want to find.
    ///
    /// Serialized internally into a base64-encoded string before sending to
    /// the RPC server.
    #[serde(with = "serializers::tx_hash_base64")]
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

impl RequestMessage for Request {
    fn method(&self) -> Method {
        Method::Tx
    }
}

impl<S: Dialect> crate::Request<S> for Request {
    type Response = DialectResponse<S::Event>;
}

impl<S: Dialect> crate::SimpleRequest<S> for Request {
    type Output = Response;
}

#[derive(Clone, Debug, Serialize)]
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
    pub tx: Vec<u8>,
    pub proof: Option<tx::Proof>,
}

/// RPC dialect helper for serialization of the response.
#[derive(Debug, Deserialize, Serialize)]
pub struct DialectResponse<Ev> {
    /// The hash of the transaction.
    ///
    /// Deserialized from a hex-encoded string (there is a discrepancy between
    /// the format used for the request and the format used for the response in
    /// the Tendermint RPC).
    pub hash: Hash,
    pub height: block::Height,
    pub index: u32,
    pub tx_result: DeliverTx<Ev>,
    #[serde(with = "serializers::bytes::base64string")]
    pub tx: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<tx::Proof>,
}

impl<Ev> crate::Response for DialectResponse<Ev> where Ev: Serialize + DeserializeOwned {}

impl<Ev> From<DialectResponse<Ev>> for Response
where
    Ev: Into<abci::Event>,
{
    fn from(msg: DialectResponse<Ev>) -> Self {
        Self {
            hash: msg.hash,
            height: msg.height,
            index: msg.index,
            tx_result: msg.tx_result.into(),
            tx: msg.tx,
            proof: msg.proof,
        }
    }
}
