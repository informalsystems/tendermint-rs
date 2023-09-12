//! Serialize and deserialize any `T` that implements [`FromStr`]
//! and [`Display`] to convert from or into string. Note this can be used for
//! all primitive data types.

use alloc::borrow::Cow;
use core::fmt::Display;
use core::str::FromStr;

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

use crate::prelude::*;

/// Deserialize string into T
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    <Cow<'_, str>>::deserialize(deserializer)?
        .parse::<T>()
        .map_err(D::Error::custom)
}

/// Serialize from T into string
pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Display,
{
    value.to_string().serialize(serializer)
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use core::convert::Infallible;
    use core::str::FromStr;
    use serde::Deserialize;

    struct ParsedStr(String);

    impl FromStr for ParsedStr {
        type Err = Infallible;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self(s.to_owned()))
        }
    }

    #[derive(Deserialize)]
    struct Foo {
        #[serde(with = "super")]
        msg: ParsedStr,
    }

    #[test]
    fn can_deserialize_owned() {
        const TEST_JSON: &str = r#"{ "msg": "\"Hello\"" }"#;
        let v = serde_json::from_str::<Foo>(TEST_JSON).unwrap();
        assert_eq!(v.msg.0, "\"Hello\"");
    }
}
