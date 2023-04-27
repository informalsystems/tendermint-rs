//! Block size parameters

use serde::{Deserialize, Serialize};

use crate::serializers;

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
    pub const fn default_time_iota_ms() -> i64 {
        1000
    }
}

mod v0_34 {
    use super::Size;
    use crate::error::Error;
    use tendermint_proto::v0_34::{
        abci::BlockParams as RawAbciSize, types::BlockParams as RawSize,
    };
    use tendermint_proto::Protobuf;

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
                time_iota_ms: value.time_iota_ms,
            })
        }
    }

    impl From<Size> for RawSize {
        fn from(value: Size) -> Self {
            // Todo: make the struct more robust so this can become infallible.
            RawSize {
                max_bytes: value.max_bytes as i64,
                max_gas: value.max_gas,
                time_iota_ms: value.time_iota_ms,
            }
        }
    }

    impl Protobuf<RawAbciSize> for Size {}

    impl TryFrom<RawAbciSize> for Size {
        type Error = Error;

        fn try_from(value: RawAbciSize) -> Result<Self, Self::Error> {
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

    impl From<Size> for RawAbciSize {
        fn from(value: Size) -> Self {
            // Todo: make the struct more robust so this can become infallible.
            RawAbciSize {
                max_bytes: value.max_bytes as i64,
                max_gas: value.max_gas,
            }
        }
    }
}

mod v0_37 {
    use super::Size;
    use crate::error::Error;
    use tendermint_proto::v0_37::types::BlockParams as RawSize;
    use tendermint_proto::Protobuf;

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
                time_iota_ms: Size::default_time_iota_ms(),
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
}

mod v0_38 {
    use super::Size;
    use crate::error::Error;
    use tendermint_proto::v0_38::types::BlockParams as RawSize;
    use tendermint_proto::Protobuf;

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
                time_iota_ms: Size::default_time_iota_ms(),
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
}
