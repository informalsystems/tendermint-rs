//! `/broadcast_tx_commit`: only returns error if `mempool.CheckTx()` errs or
//! if we timeout waiting for tx to commit.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tendermint::{block, Hash};

use crate::dialect::{CheckTx, DeliverTx, Dialect};
use crate::{prelude::*, serializers};

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

impl<S: Dialect> crate::Request<S> for Request {
    type Response = Response<S::Event>;

    fn method(&self) -> crate::Method {
        crate::Method::BroadcastTxCommit
    }
}

impl<S: Dialect> crate::SimpleRequest<S> for Request {}

/// Response from `/broadcast_tx_commit`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response<Ev> {
    /// `CheckTx` result
    pub check_tx: CheckTx<Ev>,

    /// `DeliverTx` result
    pub deliver_tx: DeliverTx<Ev>,

    /// Transaction
    pub hash: Hash,

    /// Height
    pub height: block::Height,
}

impl<Ev> crate::Response for Response<Ev> where Ev: Serialize + DeserializeOwned {}
