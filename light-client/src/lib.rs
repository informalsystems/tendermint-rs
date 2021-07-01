#![forbid(unsafe_code)]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms,
    nonstandard_style
)]
#![doc(
    html_root_url = "https://docs.rs/tendermint-light-client/0.20.0",
    html_logo_url = "https://raw.githubusercontent.com/informalsystems/tendermint-rs/master/img/logo-tendermint-rs_3961x4001.png"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! See the `light_client` module for the main documentation.

pub mod builder;
pub mod components;
pub mod contracts;
pub mod errors;
pub mod evidence;
pub mod fork_detector;
pub mod light_client;
pub mod operations;
pub mod peer_list;
pub mod predicates;
pub mod state;
pub mod store;
pub mod supervisor;
pub mod types;

pub(crate) mod utils;

#[doc(hidden)]
mod macros;

#[doc(hidden)]
pub mod tests;
