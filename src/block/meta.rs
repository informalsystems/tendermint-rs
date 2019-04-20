//! Block metadata

use super::{Header, Id};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Block metadata
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct Meta {
    /// ID of the block
    pub block_id: Id,

    /// Header of the block
    pub header: Header,
}
