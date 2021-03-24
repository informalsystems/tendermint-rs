//! Pagination-related data structures for the Tendermint RPC.

use crate::positive_number::PositiveNumber;
use crate::Error;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

/// Pagination control for those RPC client methods supporting pagination.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Paging {
    /// No explicit options set - use whatever the endpoint's defaults are.
    Default,
    /// Try to automatically fetch all pages' data.
    All,
    /// Fetch a specific page's data.
    Specific {
        /// The number of the page to fetch.
        page_number: PageNumber,
        /// The number of items to fetch per page.
        per_page: PerPage,
    },
}

/// A page number in paginated RPC responses.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct PageNumber(PositiveNumber);

impl TryFrom<usize> for PageNumber {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(value.try_into()?)
    }
}

/// The number of items to return per page, for paginated RPC responses.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct PerPage(PositiveNumber);

impl TryFrom<usize> for PerPage {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(value.try_into()?)
    }
}
