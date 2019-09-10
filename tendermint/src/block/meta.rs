//! Block metadata

use super::{Header, Id};
use serde::{Deserialize, Serialize};

/// Block metadata
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Meta {
    /// ID of the block
    pub block_id: Id,

    /// Header of the block
    pub header: Header,
}
