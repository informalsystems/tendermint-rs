//! Block size parameters

use crate::error::Error;
use core::convert::{TryFrom, TryInto};
use tendermint_proto::Protobuf;
use {
    crate::serializers,
    serde::{Deserialize, Serialize},
    tendermint_proto::types::BlockParams as RawSize,
};

/// Block size parameters
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Size {
    /// Maximum number of bytes in a block
    #[serde(with = "serializers::from_str")]
    pub max_bytes: u64,

    /// Maximum amount of gas which can be spent on a block
    #[serde(with = "serializers::from_str")]
    pub max_gas: i64,

    /// This parameter has no value anymore in Tendermint-core
    #[serde(with = "serializers::from_str", default = "Size::default_time_iota_ms")]
    pub time_iota_ms: i64,
}

impl Size {
    /// The default value for the `time_iota_ms` parameter.
    pub fn default_time_iota_ms() -> i64 {
        1000
    }
}

impl Protobuf<RawSize> for Size {}

impl TryFrom<RawSize> for Size {
    type Error = Error;

    fn try_from(value: RawSize) -> Result<Self, Self::Error> {
        Ok(Self {
            max_bytes: value
                .max_bytes
                .try_into()
                .map_err(Error::integer_overflow)?,
            max_gas: value.max_gas,
            time_iota_ms: Self::default_time_iota_ms(),
        })
    }
}

impl From<Size> for RawSize {
    fn from(value: Size) -> Self {
        // Todo: make the struct more robust so this can become infallible.
        RawSize {
            max_bytes: value.max_bytes as i64,
            max_gas: value.max_gas,
        }
    }
}
