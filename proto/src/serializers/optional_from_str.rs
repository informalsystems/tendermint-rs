//! De/serialize an optional type that must be converted from/to a string.

use core::{fmt::Display, str::FromStr};

use serde::{de::Error, Deserialize, Deserializer, Serializer};

use crate::prelude::*;
use crate::serializers::cow_str::CowStr;

pub fn serialize<S, T>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToString,
{
    match value {
        Some(t) => serializer.serialize_some(&t.to_string()),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: Display,
{
    let s = match Option::<CowStr>::deserialize(deserializer)? {
        Some(s) => s,
        None => return Ok(None),
    };
    Ok(Some(s.parse().map_err(D::Error::custom)?))
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use crate::prelude::*;
    use alloc::borrow::ToOwned;
    use core::convert::Infallible;
    use core::str::FromStr;
    use serde::Deserialize;

    #[derive(Debug, PartialEq)]
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
        msg: Option<ParsedStr>,
    }

    #[test]
    fn can_deserialize_owned() {
        const TEST_JSON: &str = r#"{ "msg": "\"Hello\"" }"#;
        let v = serde_json::from_str::<Foo>(TEST_JSON).unwrap();
        assert_eq!(v.msg, Some(ParsedStr("\"Hello\"".into())));
    }
}
