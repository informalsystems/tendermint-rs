//! Integration tests for the `yubihsm` subcommands

// This test requires USB access to a YubiHSM2
#[cfg(not(feature = "yubihsm-mock"))]
mod detect;
mod keys;
