//! `/broadcast_tx_commit`: only returns error if `mempool.CheckTx()` errs or
//! if we timeout waiting for tx to commit.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use tendermint::{abci, block, Hash};

use crate::dialect::{self, Dialect};
use crate::{prelude::*, request::RequestMessage, serializers};

/// `/broadcast_tx_commit`: only returns error if `mempool.CheckTx()` errs or
/// if we timeout waiting for tx to commit.
///
/// If `CheckTx` or `DeliverTx` fail, no error will be returned, but the
/// returned result will contain a non-OK ABCI code.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Transaction to broadcast
    #[serde(with = "serializers::bytes::base64string")]
    pub tx: Vec<u8>,
}

impl Request {
    /// Create a new commit transaction broadcast RPC request
    pub fn new(tx: impl Into<Vec<u8>>) -> Request {
        Request { tx: tx.into() }
    }
}

impl RequestMessage for Request {
    fn method(&self) -> crate::Method {
        crate::Method::BroadcastTxCommit
    }
}

impl<S: Dialect> crate::Request<S> for Request {
    type Response = DialectResponse<S::Event>;
}

impl<S: Dialect> crate::SimpleRequest<S> for Request {
    type Output = Response;
}

/// Response from `/broadcast_tx_commit`.
#[derive(Clone, Debug, Serialize)]
pub struct Response {
    /// `CheckTx` result
    pub check_tx: abci::response::CheckTx,

    /// `DeliverTx` result
    pub deliver_tx: abci::response::DeliverTx,

    /// Transaction
    pub hash: Hash,

    /// Height
    pub height: block::Height,
}

/// RPC dialect helper for serialization of the response.
#[derive(Debug, Deserialize, Serialize)]
pub struct DialectResponse<Ev> {
    /// `CheckTx` result
    pub check_tx: dialect::CheckTx<Ev>,

    /// `DeliverTx` result
    pub deliver_tx: dialect::DeliverTx<Ev>,

    /// Transaction
    pub hash: Hash,

    /// Height
    pub height: block::Height,
}

impl<Ev> crate::Response for DialectResponse<Ev> where Ev: Serialize + DeserializeOwned {}

impl<Ev> From<DialectResponse<Ev>> for Response
where
    Ev: Into<abci::Event>,
{
    fn from(msg: DialectResponse<Ev>) -> Self {
        Self {
            check_tx: msg.check_tx.into(),
            deliver_tx: msg.deliver_tx.into(),
            hash: msg.hash,
            height: msg.height,
        }
    }
}
