// Original code from <https://github.com/influxdata/pbjson/blob/main/pbjson-types/src/any.rs>
// Copyright 2022 Dan Burkert & Tokio Contributors

use prost::{DecodeError, EncodeError, Message, Name};
use subtle_encoding::base64;

use crate::prelude::*;

use super::type_url::{type_url_for, TypeUrl};
use super::PACKAGE;

/// `Any` contains an arbitrary serialized protocol buffer message along with a
/// URL that describes the type of the serialized message.
///
/// Protobuf library provides support to pack/unpack Any values in the form
/// of utility functions or additional generated methods of the Any type.
///
/// # Example
///
/// Pack and unpack a message in Rust:
///
/// ```rust,ignore
/// let foo1 = Foo { ... };
/// let any = Any::from_msg(&foo1)?;
/// let foo2 = any.to_msg::<Foo>()?;
/// assert_eq!(foo1, foo2);
/// ```
///
/// The pack methods provided by protobuf library will by default use
/// 'type.googleapis.com/full.type.name' as the type URL and the unpack
/// methods only use the fully qualified type name after the last '/'
/// in the type URL, for example "foo.bar.com/x/y.z" will yield type
/// name "y.z".
///
/// # JSON
///
/// JSON serialization of Any cannot be made compatible with the specification.
/// See <https://github.com/influxdata/pbjson/issues/2> for more information.
///
/// At the moment, an `Any` struct will be serialized as a JSON object with two fields:
/// - `typeUrl` (string): the type URL of the message
/// - `value` (string): the base64-encoded serialized message
///
/// For example:
/// ```json
/// {
///    "typeUrl": "type.googleapis.com/google.protobuf.Duration",
///    "value": "Cg0KB2NvcnA="
/// }
/// ```
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
#[cfg_attr(feature = "json-schema", derive(::schemars::JsonSchema))]
pub struct Any {
    /// A URL/resource name that uniquely identifies the type of the serialized
    /// protocol buffer message. This string must contain at least
    /// one "/" character. The last segment of the URL's path must represent
    /// the fully qualified name of the type (as in
    /// `path/google.protobuf.Duration`). The name should be in a canonical form
    /// (e.g., leading "." is not accepted).
    ///
    /// In practice, teams usually precompile into the binary all types that they
    /// expect it to use in the context of Any. However, for URLs which use the
    /// scheme `http`, `https`, or no scheme, one can optionally set up a type
    /// server that maps type URLs to message definitions as follows:
    ///
    /// * If no scheme is provided, `https` is assumed.
    /// * An HTTP GET on the URL must yield a \[google.protobuf.Type\]\[\]
    ///   value in binary format, or produce an error.
    /// * Applications are allowed to cache lookup results based on the
    ///   URL, or have them precompiled into a binary to avoid any
    ///   lookup. Therefore, binary compatibility needs to be preserved
    ///   on changes to types. (Use versioned type names to manage
    ///   breaking changes.)
    ///
    /// Note: this functionality is not currently available in the official
    /// protobuf release, and it is not used for type URLs beginning with
    /// type.googleapis.com.
    ///
    /// Schemes other than `http`, `https` (or the empty scheme) might be
    /// used with implementation specific semantics.
    #[prost(string, tag = "1")]
    pub type_url: ::prost::alloc::string::String,
    /// Must be a valid serialized protocol buffer of the above specified type.
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}

impl Any {
    /// Serialize the given message type `M` as [`Any`].
    pub fn from_msg<M>(msg: &M) -> Result<Self, EncodeError>
    where
        M: Name,
    {
        let type_url = M::type_url();
        let mut value = Vec::new();
        Message::encode(msg, &mut value)?;
        Ok(Any { type_url, value })
    }

    /// Decode the given message type `M` from [`Any`], validating that it has
    /// the expected type URL.
    pub fn to_msg<M>(&self) -> Result<M, DecodeError>
    where
        M: Default + Name + Sized,
    {
        let expected_type_url = M::type_url();

        if let (Some(expected), Some(actual)) = (
            TypeUrl::new(&expected_type_url),
            TypeUrl::new(&self.type_url),
        ) {
            if expected == actual {
                return M::decode(self.value.as_slice());
            }
        }

        let mut err = DecodeError::new(format!(
            "expected type URL: \"{}\" (got: \"{}\")",
            expected_type_url, &self.type_url
        ));
        err.push("unexpected type URL", "type_url");
        Err(err)
    }
}

impl Name for Any {
    const PACKAGE: &'static str = PACKAGE;
    const NAME: &'static str = "Any";

    fn type_url() -> String {
        type_url_for::<Self>()
    }
}

impl serde::Serialize for Any {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.type_url.is_empty() {
            len += 1;
        }
        if !self.value.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.protobuf.Any", len)?;
        if !self.type_url.is_empty() {
            struct_ser.serialize_field("typeUrl", &self.type_url)?;
        }
        if !self.value.is_empty() {
            // NOTE: A base64 string is always valid UTF-8.
            struct_ser.serialize_field(
                "value",
                &String::from_utf8_lossy(&base64::encode(&self.value)),
            )?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Any {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["type_url", "typeUrl", "value"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TypeUrl,
            Value,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> core::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(
                        &self,
                        formatter: &mut core::fmt::Formatter<'_>,
                    ) -> core::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> core::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "typeUrl" | "type_url" => Ok(GeneratedField::TypeUrl),
                            "value" => Ok(GeneratedField::Value),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Any;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct google.protobuf.Any")
            }

            fn visit_map<V>(self, mut map_: V) -> core::result::Result<Any, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut type_url__ = None;
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TypeUrl => {
                            if type_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("typeUrl"));
                            }
                            type_url__ = Some(map_.next_value()?);
                        },
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            let b64_str = map_.next_value::<String>()?;
                            let value = base64::decode(b64_str.as_bytes()).map_err(|e| {
                                serde::de::Error::custom(format!("base64 decode error: {e}"))
                            })?;
                            value__ = Some(value);
                        },
                    }
                }
                Ok(Any {
                    type_url: type_url__.unwrap_or_default(),
                    value: value__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.protobuf.Any", FIELDS, GeneratedVisitor)
    }
}

#[cfg(any(feature = "borsh", feature = "parity-scale-codec"))]
mod sealed {
    use super::Any;

    use alloc::string::String;
    use alloc::vec::Vec;

    #[cfg_attr(
        feature = "parity-scale-codec",
        derive(
            parity_scale_codec::Encode,
            parity_scale_codec::Decode,
            scale_info::TypeInfo
        )
    )]
    #[cfg_attr(
        feature = "borsh",
        derive(borsh::BorshSerialize, borsh::BorshDeserialize)
    )]
    struct InnerAny {
        pub type_url: String,
        pub value: Vec<u8>,
    }

    #[cfg(feature = "borsh")]
    impl borsh::BorshSerialize for Any {
        fn serialize<W: borsh::io::Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
            let inner_any = InnerAny {
                type_url: self.type_url.clone(),
                value: self.value.clone(),
            };

            borsh::BorshSerialize::serialize(&inner_any, writer)
        }
    }

    #[cfg(feature = "borsh")]
    impl borsh::BorshDeserialize for Any {
        fn deserialize_reader<R: borsh::io::Read>(reader: &mut R) -> borsh::io::Result<Self> {
            let inner_any = InnerAny::deserialize_reader(reader)?;

            Ok(Any {
                type_url: inner_any.type_url,
                value: inner_any.value,
            })
        }
    }

    #[cfg(feature = "parity-scale-codec")]
    impl parity_scale_codec::Encode for Any {
        fn encode_to<T: parity_scale_codec::Output + ?Sized>(&self, writer: &mut T) {
            let inner_any = InnerAny {
                type_url: self.type_url.clone(),
                value: self.value.clone(),
            };
            inner_any.encode_to(writer);
        }
    }
    #[cfg(feature = "parity-scale-codec")]
    impl parity_scale_codec::Decode for Any {
        fn decode<I: parity_scale_codec::Input>(
            input: &mut I,
        ) -> Result<Self, parity_scale_codec::Error> {
            let inner_any = InnerAny::decode(input)?;
            Ok(Any {
                type_url: inner_any.type_url.clone(),
                value: inner_any.value,
            })
        }
    }

    #[cfg(feature = "parity-scale-codec")]
    impl scale_info::TypeInfo for Any {
        type Identity = Self;

        fn type_info() -> scale_info::Type {
            scale_info::Type::builder()
                .path(scale_info::Path::new("Any", "ibc_proto::google::protobuf"))
                // i128 is chosen before we represent the timestamp is nanoseconds, which is represented as a i128 by Time
                .composite(
                    scale_info::build::Fields::named()
                        .field(|f| f.ty::<String>().name("type_url").type_name("String"))
                        .field(|f| f.ty::<Vec<u8>>().name("value").type_name("Vec<u8>")),
                )
        }
    }
}
