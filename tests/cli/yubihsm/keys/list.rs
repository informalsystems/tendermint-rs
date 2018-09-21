//! Integration tests for the `yubihsm keys list` subcommand

use cli;

#[test]
fn keys_command_test() {
    #[allow(unused_mut)]
    let mut args = vec!["yubihsm", "keys", "list"];

    #[cfg(feature = "yubihsm-mock")]
    args.extend_from_slice(&["-c", "tests/cli/yubihsm/kms-mockhsm.toml"]);

    // TODO: parse results
    cli::run_successfully(args.as_slice());
}
