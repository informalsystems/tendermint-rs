//! `/block` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};
use tendermint::block::{self, Block};

use crate::dialect::{self, Dialect};
use crate::request::RequestMessage;

/// Get information about a specific block
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Height of the block to request.
    ///
    /// If no height is provided, it will fetch results for the latest block.
    pub height: Option<block::Height>,
}

impl Request {
    /// Create a new request for information about a particular block
    pub fn new(height: block::Height) -> Self {
        Self {
            height: Some(height),
        }
    }
}

impl RequestMessage for Request {
    fn method(&self) -> crate::Method {
        crate::Method::Block
    }
}

impl crate::Request<dialect::v0_34::Dialect> for Request {
    type Response = Response;
}

impl crate::Request<dialect::v0_37::Dialect> for Request {
    type Response = Response;
}

impl crate::Request<dialect::v0_38::Dialect> for Request {
    type Response = self::v0_38::DialectResponse;
}

impl<S: Dialect> crate::SimpleRequest<S> for Request
where
    Self: crate::Request<S>,
    Response: From<Self::Response>,
{
    type Output = Response;
}

/// Block responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Block ID
    pub block_id: block::Id,

    /// Block data
    pub block: Block,
}

impl crate::Response for Response {}

pub mod v0_38 {
    use std::vec::Vec;

    use block::{Commit, Header};
    use tendermint::evidence;

    use tendermint_proto::v0_38::types::Block as RawBlock;

    use super::*;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct DialectResponse {
        /// Block ID
        pub block_id: block::Id,

        /// Block data
        pub block: DialectBlock,
    }

    impl crate::Response for DialectResponse {}

    impl From<DialectResponse> for Response {
        fn from(msg: DialectResponse) -> Self {
            Self {
                block_id: msg.block_id,
                block: msg.block.into(),
            }
        }
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(try_from = "RawBlock", into = "RawBlock")]
    pub struct DialectBlock {
        /// Block header
        pub header: Header,

        /// Transaction data
        pub data: Vec<Vec<u8>>,

        /// Evidence of malfeasance
        pub evidence: evidence::List,

        /// Last commit, should be `None` for the initial block.
        pub last_commit: Option<Commit>,
    }

    impl From<DialectBlock> for RawBlock {
        fn from(msg: DialectBlock) -> Self {
            RawBlock::from(Block::from(msg))
        }
    }

    impl TryFrom<RawBlock> for DialectBlock {
        type Error = <Block as TryFrom<RawBlock>>::Error;

        fn try_from(value: RawBlock) -> Result<Self, Self::Error> {
            Block::try_from(value).map(DialectBlock::from)
        }
    }

    impl From<DialectBlock> for Block {
        fn from(msg: DialectBlock) -> Self {
            Self::new(msg.header, msg.data, msg.evidence, msg.last_commit)
        }
    }

    impl From<Block> for DialectBlock {
        fn from(msg: Block) -> Self {
            Self {
                header: msg.header,
                data: msg.data,
                evidence: msg.evidence,
                last_commit: msg.last_commit,
            }
        }
    }
}
