//! tendermint-proto library gives the developer access to the Tendermint proto-defined structs.

#![deny(warnings, trivial_casts, trivial_numeric_casts, unused_import_braces)]
#![allow(clippy::large_enum_variant)]
#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/tendermint-proto/0.17.0-rc2")]

// Built-in prost_types with slight customization to enable JSON-encoding
pub mod google {
    pub mod protobuf {
        include!("prost/google.protobuf.rs");
        // custom Timeout and Duration types that have valid doctest documentation texts
        include!("protobuf.rs");
    }
}

mod tendermint;
pub use tendermint::*;

mod domaintype;
pub use domaintype::DomainType;

mod error;
pub use error::{Error, Kind};

pub mod serializers;
