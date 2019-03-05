//! Integration tests for the `yubihsm keys export` subcommand

use crate::cli;

#[test]
fn keys_export_command_test() {
    #[allow(unused_mut)]
    let mut args = vec!["yubihsm", "keys", "export", "1"];

    #[cfg(feature = "yubihsm-mock")]
    args.extend_from_slice(&["-c", super::KMS_CONFIG_PATH]);

    cli::run_successfully(args.as_slice());
}
