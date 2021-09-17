//! Ordering of paginated RPC responses.

use crate::prelude::*;
use crate::Error;
use core::str::FromStr;
use serde::{Deserialize, Serialize};

/// Ordering of paginated RPC responses.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Order {
    /// Ascending order
    #[serde(rename = "asc")]
    Ascending,

    /// Descending order
    #[serde(rename = "desc")]
    Descending,
}

impl FromStr for Order {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" => Ok(Self::Ascending),
            "desc" => Ok(Self::Descending),
            _ => Err(Error::invalid_params(format!(
                "invalid order type: {} (must be \"asc\" or \"desc\")",
                s
            ))),
        }
    }
}
