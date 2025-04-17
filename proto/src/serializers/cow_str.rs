//! Wrapper `Cow<'_, str>` for deserializing without allocation.
//!
//! This is a workaround for [serde's issue 1852](https://github.com/serde-rs/serde/issues/1852).

use alloc::borrow::{Cow, ToOwned};
use alloc::string::String;
use core::fmt::{self, Debug, Display, Formatter};
use core::ops::Deref;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

/// Wrapper `Cow<'_, str>` for deserializing without allocation.
#[derive(Default)]
pub struct CowStr<'a>(Cow<'a, str>);

impl<'a> CowStr<'a> {
    /// Convert into `Cow<'a, str>`.
    pub fn into_inner(self) -> Cow<'a, str> {
        self.0
    }
}

impl<'de> Deserialize<'de> for CowStr<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = CowStr<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(CowStr(Cow::Borrowed(value)))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(CowStr(Cow::Owned(value.to_owned())))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(CowStr(Cow::Owned(value)))
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl Serialize for CowStr<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl Debug for CowStr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <&str as Debug>::fmt(&&*self.0, f)
    }
}

impl Display for CowStr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <&str as Display>::fmt(&&*self.0, f)
    }
}

impl Deref for CowStr<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for CowStr<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<[u8]> for CowStr<'_> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl<'a> From<CowStr<'a>> for Cow<'a, str> {
    fn from(value: CowStr<'a>) -> Self {
        value.0
    }
}

impl<'a> From<Cow<'a, str>> for CowStr<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        CowStr(value)
    }
}

/// Serialize `Cow<'_, str>`.
#[allow(clippy::ptr_arg)]
pub fn serialize<S>(value: &Cow<'_, str>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(value)
}

/// Deserialize `Cow<'_, str>`.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Cow<'de, str>, D::Error>
where
    D: Deserializer<'de>,
{
    CowStr::deserialize(deserializer).map(|value| value.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed() {
        struct Test(u32);

        impl<'de> Deserialize<'de> for Test {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let s = CowStr::deserialize(deserializer)?;
                assert!(matches!(s.0, Cow::Borrowed(_)));
                Ok(Test(s.parse().unwrap()))
            }
        }

        let v = serde_json::from_str::<Test>("\"2\"").unwrap();
        assert_eq!(v.0, 2);
    }

    #[test]
    fn owned() {
        struct Test(u32);

        impl<'de> Deserialize<'de> for Test {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let s = CowStr::deserialize(deserializer)?;
                assert!(matches!(s.0, Cow::Owned(_)));
                Ok(Test(s.parse().unwrap()))
            }
        }

        let json_value = serde_json::from_str::<serde_json::Value>("\"2\"").unwrap();
        let v = serde_json::from_value::<Test>(json_value).unwrap();
        assert_eq!(v.0, 2);
    }
}
