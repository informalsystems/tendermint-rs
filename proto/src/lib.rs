//! tendermint-proto library gives the developer access to the Tendermint proto-defined structs.

// This module setup is necessary because the generated code contains "super::" calls for
// dependencies. Unfortunately, prost doesn't create this for us automatically.

#![deny(
    warnings,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces
)]
#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/tendermint-proto/0.1.0")]

mod tendermint {
    pub mod abci {
        #![allow(missing_docs)]
        #![allow(clippy::large_enum_variant)]
        include!("prost/tendermint.abci.rs");
    }
    pub mod blockchain {
        #![allow(missing_docs)]
        #![allow(clippy::large_enum_variant)]
        include!("prost/tendermint.blockchain.rs");
    }
    pub mod consensus {
        #![allow(missing_docs)]
        include!("prost/tendermint.consensus.rs");
    }
    pub mod crypto {
        #![allow(missing_docs)]
        include!("prost/tendermint.crypto.rs");
    }
    pub mod evidence {
        #![allow(missing_docs)]
        include!("prost/tendermint.evidence.rs");
    }
    pub mod libs {
        #![allow(missing_docs)]
        pub mod bits {
            #![allow(missing_docs)]
            include!("prost/tendermint.libs.bits.rs");
        }
    }
    pub mod mempool {
        #![allow(missing_docs)]
        include!("prost/tendermint.mempool.rs");
    }
    pub mod p2p {
        #![allow(missing_docs)]
        include!("prost/tendermint.p2p.rs");
    }
    pub mod privval {
        #![allow(missing_docs)]
        include!("prost/tendermint.privval.rs");
    }
    pub mod rpc {
        #![allow(missing_docs)]
        pub mod grpc {
            #![allow(missing_docs)]
            include!("prost/tendermint.rpc.grpc.rs");
        }
    }
    pub mod state {
        #![allow(missing_docs)]
        include!("prost/tendermint.state.rs");
    }
    pub mod statesync {
        #![allow(missing_docs)]
        include!("prost/tendermint.statesync.rs");
    }
    pub mod store {
        #![allow(missing_docs)]
        include!("prost/tendermint.store.rs");
    }
    pub mod types {
        #![allow(missing_docs)]
        #![allow(clippy::large_enum_variant)]
        include!("prost/tendermint.types.rs");
    }
    pub mod version {
        #![allow(missing_docs)]
        include!("prost/tendermint.version.rs");
    }
}

pub use tendermint::*;
