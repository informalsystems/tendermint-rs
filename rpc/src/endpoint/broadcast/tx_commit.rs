//! `/broadcast_tx_commit`: only returns error if `mempool.CheckTx()` errs or
//! if we timeout waiting for tx to commit.

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

/// Response from `/broadcast_tx_commit`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    /// `CheckTx` result
    pub check_tx: abci::response::CheckTx,

    /// Result of executing the transaction.
    ///
    /// The JSON field carrying this data is named `deliver_tx` in
    /// CometBFT versions before 0.38.
    #[serde(alias = "deliver_tx")]
    pub tx_result: abci::types::ExecTxResult,

    /// Transaction
    pub hash: Hash,

    /// Height
    pub height: block::Height,
}

impl crate::Response for Response {}

/// Serialization for /broadcast_tx_commit endpoint format in Tendermint 0.34
pub mod v0_34 {
    use super::Response;
    use crate::dialect;
    use crate::dialect::v0_34::Event;
    use serde::{Deserialize, Serialize};
    use tendermint::{block, Hash};

    /// RPC dialect helper for serialization of the response.
    #[derive(Debug, Deserialize, Serialize)]
    pub struct DialectResponse {
        /// `CheckTx` result
        pub check_tx: dialect::CheckTx<Event>,

        /// `DeliverTx` result
        pub deliver_tx: dialect::DeliverTx<Event>,

        /// Transaction
        pub hash: Hash,

        /// Height
        pub height: block::Height,
    }

    impl crate::Response for DialectResponse {}

    impl From<DialectResponse> for Response {
        fn from(msg: DialectResponse) -> Self {
            Self {
                check_tx: msg.check_tx.into(),
                tx_result: msg.deliver_tx.into(),
                hash: msg.hash,
                height: msg.height,
            }
        }
    }
}

/// Serialization for /broadcast_tx_commit endpoint format in CometBFT 0.37
pub mod v0_37 {
    use super::Response;
    use crate::dialect;
    use serde::{Deserialize, Serialize};
    use tendermint::{abci::Event, block, Hash};

    /// RPC dialect helper for serialization of the response.
    #[derive(Debug, Deserialize, Serialize)]
    pub struct DialectResponse {
        /// `CheckTx` result
        pub check_tx: dialect::CheckTx<Event>,

        /// `DeliverTx` result
        pub deliver_tx: dialect::DeliverTx<Event>,

        /// Transaction
        pub hash: Hash,

        /// Height
        pub height: block::Height,
    }

    impl crate::Response for DialectResponse {}

    impl From<DialectResponse> for Response {
        fn from(msg: DialectResponse) -> Self {
            Self {
                check_tx: msg.check_tx.into(),
                tx_result: msg.deliver_tx.into(),
                hash: msg.hash,
                height: msg.height,
            }
        }
    }
}
