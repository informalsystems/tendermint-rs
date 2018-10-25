//! Integration tests for the `yubihsm` subcommands

/// Path to KMS configuration file for `yubihsm::MockHSM`-based testing
#[allow(dead_code)]
pub const KMS_CONFIG_PATH: &str = "tests/support/kms_yubihsm_mock.toml";

// This test requires USB access to a YubiHSM2
#[cfg(not(feature = "yubihsm-mock"))]
mod detect;
mod keys;
