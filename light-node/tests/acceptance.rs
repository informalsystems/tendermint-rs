//! Acceptance test: runs the application as a subprocess and asserts its
//! output for given argument combinations matches what is expected.
//!
//! Modify and/or delete these as you see fit to test the specific needs of
//! your application.
//!
//! For more information, see:
//! <https://docs.rs/abscissa_core/latest/abscissa_core/testing/index.html>

// Tip: Deny warnings with `RUSTFLAGS="-D warnings"` environment variable in CI

#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    rust_2018_idioms,
    trivial_casts,
    unused_lifetimes,
    unused_qualifications
)]

use abscissa_core::testing::prelude::*;
use once_cell::sync::Lazy;

/// Executes your application binary via `cargo run`.
///
/// Storing this value as a [`Lazy`] static ensures that all instances of
/// the runner acquire a mutex when executing commands and inspecting
/// exit statuses, serializing what would otherwise be multithreaded
/// invocations as `cargo test` executes tests in parallel by default.
pub static RUNNER: Lazy<CmdRunner> = Lazy::new(CmdRunner::default);

/// Use `LightNodeConfig::default()` value if no config or args
#[test]
#[ignore]
fn start_no_args() {
    let mut runner = RUNNER.clone();
    let mut cmd = runner.arg("start").capture_stdout().run();
    cmd.stdout().expect_line("");
    cmd.wait().unwrap().expect_success();
}

/// Use command-line argument value
#[test]
#[ignore]
fn start_with_args() {
    todo!()
}

/// Use configured value
#[test]
#[ignore]
fn start_with_config_no_args() {
    todo!()
}

/// Override configured value with command-line argument
#[test]
#[ignore]
fn start_with_config_and_args() {
    todo!()
}

/// Example of a test which matches a regular expression
#[test]
#[ignore] // Ignored because of https://github.com/informalsystems/tendermint-rs/issues/560
fn version_no_args() {
    let mut runner = RUNNER.clone();
    let mut cmd = runner.arg("version").capture_stdout().run();
    cmd.stdout().expect_regex(r"\A[\w-]+ [\d\.\-]+\z");
}