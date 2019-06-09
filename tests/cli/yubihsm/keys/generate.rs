//! Integration tests for the `yubihsm keys generate` subcommand

use crate::cli;
use std::str;

#[test]
fn keys_generate_command_test() {
    #[allow(unused_mut)]
    let mut args = vec!["yubihsm", "keys", "generate", "1"];

    #[cfg(feature = "yubihsm-mock")]
    args.extend_from_slice(&["-c", super::KMS_CONFIG_PATH]);

    let cmd_out = cli::run_successfully(args.as_slice());
    assert_eq!(true, cmd_out.status.success());
    assert_eq!(true, cmd_out.stderr.is_empty());

    let stdout = str::from_utf8(&cmd_out.stdout).unwrap().trim().to_owned();
    assert!(stdout.contains("Generated"));
    assert!(stdout.contains("key 0x0001"));
}
