//! Types that facilitate whole, positive numbers.

use crate::Error as RpcError;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use std::convert::{TryFrom, TryInto};

/// A whole, positive number >= 1.
#[derive(Debug, Clone, Copy, Serialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct PositiveNumber(usize);

impl PositiveNumber {
    fn new(value: usize) -> Result<Self, RpcError> {
        if value >= 1 {
            Ok(Self(value))
        } else {
            Err(RpcError::client_internal_error("value must be >= 1"))
        }
    }
}

impl TryFrom<usize> for PositiveNumber {
    type Error = RpcError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PositiveNumber> for usize {
    fn from(n: PositiveNumber) -> Self {
        n.0
    }
}

impl<'de> Deserialize<'de> for PositiveNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = i64::deserialize(deserializer)?;
        let raw_usize: usize = raw
            .try_into()
            .map_err(|_| D::Error::custom(format!("value out of range: {}", raw)))?;
        PositiveNumber::try_from(raw_usize).map_err(|e: RpcError| D::Error::custom(e.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const JSON_POSITIVE: &str = r#"{"num": 1}"#;
    const JSON_NEGATIVE: &str = r#"{"num": -1}"#;
    const JSON_NON_POSITIVE: &str = r#"{"num": 0}"#;

    #[derive(Debug, Deserialize)]
    struct TestStruct {
        num: PositiveNumber,
    }

    #[test]
    fn positive_numbers() {
        let n = PositiveNumber::try_from(1).unwrap();
        assert_eq!(usize::from(n), 1);
    }

    #[test]
    fn non_positive_numbers() {
        let r = PositiveNumber::try_from(0);
        assert!(r.is_err());
    }

    #[test]
    fn deserialize_positive() {
        let s = serde_json::from_str::<TestStruct>(JSON_POSITIVE).unwrap();
        assert_eq!(usize::from(s.num), 1);
    }

    #[test]
    fn deserialize_negative() {
        let r = serde_json::from_str::<TestStruct>(JSON_NEGATIVE);
        assert!(r.is_err());
    }

    #[test]
    fn deserialize_non_positive() {
        let r = serde_json::from_str::<TestStruct>(JSON_NON_POSITIVE);
        assert!(r.is_err());
    }
}
