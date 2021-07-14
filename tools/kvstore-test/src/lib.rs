//! The kvstore-test crate enables online testing against a kvstore endpoint.
//! Option 1: there is an existing kvstore RPC endpoint at 127.0.0.1:26657
//!           for example in the CI setup this has been taken care of.
//! Run:
//!     cargo test
//! This will execute all integration tests against the RPC endpoint.
//!
//! Option 2: the docker daemon is installed and accessible on the machine where the test will
//! happen           for example: on a developer machine
//! Run:
//!     cargo make
//! This will start a docker container with Tendermint and attach port 26657 to the host machine.
//! Then it will run all tests against the freshly created endpoint.
//! Make sure you installed cargo-make by running `cargo install cargo-make` first.
