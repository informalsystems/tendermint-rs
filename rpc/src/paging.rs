//! Pagination-related data structures for the Tendermint RPC.

use crate::{error, Error};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::str::FromStr;

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
pub struct PageNumber(usize);

impl FromStr for PageNumber {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw = i64::from_str(s).map_err(error::parse_int_error)?;
        let raw_usize: usize = raw.try_into().map_err(error::out_of_range_error)?;
        Ok(raw_usize.into())
    }
}

impl std::fmt::Display for PageNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for PageNumber {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// The number of items to return per page, for paginated RPC responses.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct PerPage(u8);

impl FromStr for PerPage {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw = i64::from_str(s).map_err(error::parse_int_error)?;
        let raw_u8: u8 = raw.try_into().map_err(error::out_of_range_error)?;
        Ok(raw_u8.into())
    }
}

impl std::fmt::Display for PerPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u8> for PerPage {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
