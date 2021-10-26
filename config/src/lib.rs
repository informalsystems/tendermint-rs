//! Tendermint Configuration Utilities
//!
//! This crate defines the [`TendermintConfig`] type, which is used by
//! crates such as `tendermint-rpc` to perform operations based on
//! a common configuration type.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod net;

mod config;
mod error;
mod node_key;
mod prelude;
mod priv_validator_key;

pub use config::*;
pub use error::*;
pub use node_key::NodeKey;
pub use priv_validator_key::PrivValidatorKey;
