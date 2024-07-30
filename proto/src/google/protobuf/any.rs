use prost::{DecodeError, EncodeError, Message, Name};

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
/// > JSON serialization of Any cannot be made compatible with the specification,
/// > and is therefore left unimplemented at the moment.
/// > See <https://github.com/influxdata/pbjson/issues/2> for more information.
///
/// The JSON representation of an `Any` value uses the regular
/// representation of the deserialized, embedded message, with an
/// additional field `@type` which contains the type URL. Example:
///
/// ```text
/// package google.profile;
/// message Person {
///    string first_name = 1;
///    string last_name = 2;
/// }
///
/// {
///    "@type": "type.googleapis.com/google.profile.Person",
///    "firstName": <string>,
///    "lastName": <string>
/// }
/// ```
///
/// If the embedded message type is well-known and has a custom JSON
/// representation, that representation will be embedded adding a field
/// `value` which holds the custom JSON in addition to the `@type`
/// field. Example (for message \[google.protobuf.Duration\]\[\]):
///
/// ```text
/// {
///    "@type": "type.googleapis.com/google.protobuf.Duration",
///    "value": "1.212s"
/// }
/// ```
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
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
