//! `tendermint-testgen` is a small utility for producing Tendermint data
//! structures from minimal input (for testing purposes only).
//!
//! See the [repository] for details.
//!
//! [repository]: https://github.com/informalsystems/tendermint-rs/tree/master/testgen

#![allow(deprecated)]

#[macro_use]
pub mod helpers;

/// Helper types for generating Tendermint datastructures
pub mod commit;
pub mod consensus;
pub mod generator;
pub mod header;
pub mod light_block;
pub mod light_chain;
pub mod time;
pub mod validator;
pub mod validator_set;
pub mod vote;

pub use commit::Commit;
pub use generator::Generator;
pub use header::Header;
pub use light_block::LightBlock;
pub use light_chain::LightChain;
pub use time::Time;
pub use validator::Validator;
pub use validator_set::ValidatorSet;
pub use vote::Vote;

/// Helpers for organizing and running the tests
pub mod apalache;
pub mod command;
pub mod jsonatr;
pub mod tester;

pub use command::Command;
pub use tester::TestEnv;
pub use tester::Tester;
