//! Integration tests for the `yubihsm keys generate` subcommand

use cli;

#[test]
fn keys_generate_command_test() {
    #[allow(unused_mut)]
    let mut args = vec!["yubihsm", "keys", "generate", "1"];

    #[cfg(feature = "yubihsm-mock")]
    args.extend_from_slice(&["-c", super::KMS_CONFIG_PATH]);

    let out = cli::run_successfully(args.as_slice());

    assert_eq!(true, out.status.success());
    assert_eq!(true, out.stderr.is_empty());
    assert_eq!(
        true,
        String::from_utf8(out.stdout)
            .unwrap()
            .trim_start()
            .starts_with("Generated key #1:")
    );
}
