//! Integration tests for the `yubihsm detect` subcommand

use crate::cli;

#[test]
fn detect_command_test() {
    // TODO: parse results
    cli::run_successfully(&["yubihsm", "detect"]);
}
