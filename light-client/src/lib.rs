#![forbid(unsafe_code)]
#![deny(
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms,
    nonstandard_style
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/informalsystems/tendermint-rs/master/img/logo-tendermint-rs_3961x4001.png"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! See the `light_client` module for the main documentation.

pub mod builder;
pub mod components;
pub mod contracts;
pub mod errors;
pub mod instance;
pub mod light_client;
pub mod misbehavior;
pub mod state;
pub mod store;

pub(crate) mod utils;

// Re-export `tendermint-light-client-verifier` crate.
pub use tendermint_light_client_verifier as verifier;
// Re-export for backward compatibility
pub use verifier::{operations, predicates, types};

#[doc(hidden)]
pub mod tests;
