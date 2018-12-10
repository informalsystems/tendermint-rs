//! Integration tests for the `yubihsm keys list` subcommand

use crate::cli;

#[test]
fn keys_command_test() {
    #[allow(unused_mut)]
    let mut args = vec!["yubihsm", "keys", "list"];

    #[cfg(feature = "yubihsm-mock")]
    args.extend_from_slice(&["-c", super::KMS_CONFIG_PATH]);

    let out = cli::run_successfully(args.as_slice());

    assert_eq!(true, out.status.success());
    assert_eq!(true, out.stdout.is_empty());
    assert_eq!(
        true,
        String::from_utf8(out.stderr)
            .unwrap()
            .trim()
            .starts_with("error: no keys in this YubiHSM")
    );
}
