//! Tests for the KMS command-line interface

use std::{
    ffi::OsStr,
    io::{self, Write},
    process::{Command, Output},
};

use super::KMS_EXE_PATH;

#[cfg(feature = "yubihsm")]
mod yubihsm;

/// Run the `tmkms` CLI command with the given arguments
pub fn run<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(KMS_EXE_PATH).args(args).output().unwrap()
}

/// Run the `tmkms` CLI command with the expectation that it will exit successfully,
/// panicking and printing stdout/stderr if it does not
pub fn run_successfully<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = run(args);
    let status_code = output.status.code().unwrap();

    if status_code == 0 {
        output
    } else {
        io::stdout().write(&output.stdout).unwrap();
        io::stderr().write(&output.stderr).unwrap();

        panic!("{} exited with error status: {}", KMS_EXE_PATH, status_code);
    }
}

#[test]
fn test_usage() {
    let status_code = run(&[] as &[&OsStr]).status.code().unwrap();
    assert_eq!(status_code, 0);
}
