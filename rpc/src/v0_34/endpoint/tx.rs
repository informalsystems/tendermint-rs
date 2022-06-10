//! `/tx` endpoint JSON-RPC wrapper
//!
//! The encoding of this endpoint's request data has been changed in
//! RPC version 0.35. The serialization schema provided here implements
//! the encoding as per version 0.34.

use serde::{Deserialize, Serialize};

use crate::v0_34::serializers;
use crate::{abci, Method};

/// Request for finding a transaction by its hash.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// The hash of the transaction we want to find.
    ///
    /// Serialized internally into a base64-encoded string before sending to
    /// the RPC server.
    #[serde(with = "serializers::hash_base64")]
    pub hash: abci::transaction::Hash,
    /// Whether or not to include the proofs of the transaction's inclusion in
    /// the block.
    pub prove: bool,
}

impl Request {
    /// Constructor.
    pub fn new(hash: abci::transaction::Hash, prove: bool) -> Self {
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

pub type Response = crate::endpoint::tx::Response;
