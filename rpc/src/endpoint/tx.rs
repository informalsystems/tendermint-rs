//! `/tx` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};
use tendermint::{abci, block, tx, Hash};

use crate::dialect::{self, Dialect};
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

impl crate::Request<dialect::v0_34::Dialect> for Request {
    type Response = self::v0_34::DialectResponse;
}

impl crate::Request<dialect::v0_37::Dialect> for Request {
    type Response = Response;
}

impl<S: Dialect> crate::SimpleRequest<S> for Request
where
    Self: crate::Request<S>,
    Response: From<Self::Response>,
{
    type Output = Response;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    /// The hash of the transaction.
    ///
    /// Deserialized from a hex-encoded string (there is a discrepancy between
    /// the format used for the request and the format used for the response in
    /// the Tendermint RPC).
    pub hash: Hash,
    pub height: block::Height,
    pub index: u32,
    pub tx_result: abci::types::ExecTxResult,
    #[serde(with = "serializers::bytes::base64string")]
    pub tx: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<tx::Proof>,
}

impl crate::Response for Response {}

/// Serialization for /tx endpoint format in Tendermint 0.34
pub mod v0_34 {
    use super::Response;
    use crate::dialect::v0_34::Event;
    use crate::prelude::*;
    use crate::{dialect, serializers};
    use serde::{Deserialize, Serialize};
    use tendermint::{block, tx, Hash};

    /// RPC dialect helper for serialization of the response.
    #[derive(Debug, Deserialize, Serialize)]
    pub struct DialectResponse {
        /// The hash of the transaction.
        ///
        /// Deserialized from a hex-encoded string (there is a discrepancy between
        /// the format used for the request and the format used for the response in
        /// the Tendermint RPC).
        pub hash: Hash,
        pub height: block::Height,
        pub index: u32,
        pub tx_result: dialect::DeliverTx<Event>,
        #[serde(with = "serializers::bytes::base64string")]
        pub tx: Vec<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub proof: Option<tx::Proof>,
    }

    impl crate::Response for DialectResponse {}

    impl From<DialectResponse> for Response {
        fn from(msg: DialectResponse) -> Self {
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
}
