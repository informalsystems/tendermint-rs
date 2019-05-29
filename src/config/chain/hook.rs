use serde::Deserialize;
use std::ffi::OsString;

/// Configuration for a particular hook to invoke
#[derive(Default, Deserialize, Debug)]
pub struct HookConfig {
    /// Command (with arguments) to invoke
    pub cmd: Vec<OsString>,

    /// Timeout (in seconds) to wait when executing the command (default 5)
    pub timeout_secs: Option<u64>,

    /// Whether or not to fail open or closed if this command fails to execute.
    /// Failing closed will prevent the KMS from starting if this command fails.
    pub fail_closed: bool,
}
