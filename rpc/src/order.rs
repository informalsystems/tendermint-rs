//! Ordering of paginated RPC responses.

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
