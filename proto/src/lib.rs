//! tendermint-proto library gives the developer access to the Tendermint proto-defined structs.

// This module setup is necessary because the generated code contains "super::" calls for
// dependencies. Unfortunately, prost doesn't create this for us automatically.

#![deny(warnings, trivial_casts, trivial_numeric_casts, unused_import_braces)]
#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/tendermint-proto/0.2.0")]

// Built-in prost_types with slight customization to enable JSON-encoding
pub mod google {
    pub mod protobuf {
        include!("prost/google.protobuf.rs");
        // custom Timeout and Duration types that have valid doctest documentation texts
        include!("protobuf.rs");
    }
}

mod tendermint {
    pub mod abci {
        #![allow(clippy::large_enum_variant)]
        include!("prost/tendermint.abci.rs");
    }
    pub mod blockchain {
        #![allow(clippy::large_enum_variant)]
        include!("prost/tendermint.blockchain.rs");
    }
    pub mod consensus {
        include!("prost/tendermint.consensus.rs");
    }
    pub mod crypto {
        include!("prost/tendermint.crypto.rs");
    }
    pub mod evidence {
        include!("prost/tendermint.evidence.rs");
    }
    pub mod libs {
        pub mod bits {
            include!("prost/tendermint.libs.bits.rs");
        }
    }
    pub mod mempool {
        include!("prost/tendermint.mempool.rs");
    }
    pub mod p2p {
        include!("prost/tendermint.p2p.rs");
    }
    pub mod privval {
        include!("prost/tendermint.privval.rs");
    }
    pub mod rpc {
        pub mod grpc {
            include!("prost/tendermint.rpc.grpc.rs");
        }
    }
    pub mod state {
        include!("prost/tendermint.state.rs");
    }
    pub mod statesync {
        include!("prost/tendermint.statesync.rs");
    }
    pub mod store {
        include!("prost/tendermint.store.rs");
    }
    pub mod types {
        #![allow(clippy::large_enum_variant)]
        include!("prost/tendermint.types.rs");
    }
    pub mod version {
        include!("prost/tendermint.version.rs");
    }
}

pub use tendermint::*;

mod domaintype;
pub use domaintype::DomainType;

mod error;
pub use error::{Error, Kind};

pub mod serializers;
