//! `/validators` endpoint JSON-RPC wrapper

use crate::prelude::*;
use crate::{PageNumber, PerPage};
use serde::{Deserialize, Serialize};
use tendermint::{block, validator};

/// The default number of validators to return per page.
pub const DEFAULT_VALIDATORS_PER_PAGE: u8 = 30;

/// List validators for a specific block
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub struct Request {
    /// The height at which to retrieve the validator set. If not specified,
    /// defaults to the latest height.
    pub height: Option<block::Height>,
    /// The number of the page to fetch.
    #[serde(with = "tendermint_proto::serializers::optional_from_str")]
    pub page: Option<PageNumber>,
    /// The number of validators to fetch per page.
    #[serde(with = "tendermint_proto::serializers::optional_from_str")]
    pub per_page: Option<PerPage>,
}

impl Request {
    /// List validators for a specific block.
    ///
    /// See the [Tendermint RPC] for the defaults for each option when set to
    /// `None`.
    ///
    /// [Tendermint RPC]: https://docs.tendermint.com/master/rpc/#/Info/validators
    pub fn new(
        height: Option<block::Height>,
        page: Option<PageNumber>,
        per_page: Option<PerPage>,
    ) -> Self {
        Self {
            height,
            page,
            per_page,
        }
    }
}

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::Validators
    }
}

impl crate::SimpleRequest for Request {}

/// Validator responses
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Response {
    /// Block height
    pub block_height: block::Height,

    /// Validator list
    pub validators: Vec<validator::Info>,

    /// Total number of validators for this block height.
    #[serde(with = "tendermint_proto::serializers::from_str")]
    pub total: i32,
}

impl crate::Response for Response {}

impl Response {
    /// Constructor.
    pub fn new(block_height: block::Height, validators: Vec<validator::Info>, total: i32) -> Self {
        Self {
            block_height,
            validators,
            total,
        }
    }
}
