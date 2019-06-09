//! State hook support: obtain `ConsensusState` from an external source

use crate::{
    config::chain::HookConfig,
    error::{Error, ErrorKind::HookError},
};
use serde::Deserialize;
use std::{process::Command, time::Duration};
use tendermint::block;
use wait_timeout::ChildExt;

/// Default timeout to use when a user one is unspecified
const DEFAULT_TIMEOUT_SECS: u64 = 1;

/// Sanity limit on how far the block height from the hook can diverge from the
/// last known state
pub const BLOCK_HEIGHT_SANITY_LIMIT: u64 = 9000;

/// Run the given hook command to obtain the last signing state
pub fn run(config: &HookConfig) -> Result<Output, Error> {
    let mut child = Command::new(&config.cmd[0])
        .args(&config.cmd[1..])
        .spawn()?;
    let timeout = Duration::from_secs(config.timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS));

    match child.wait_timeout(timeout)? {
        Some(status) => {
            if status.success() {
                if let Some(stdout) = child.stdout {
                    Ok(serde_json::from_reader(stdout)?)
                } else {
                    fail!(HookError, "couldn't consume stdout from child");
                }
            } else {
                fail!(HookError, "subcommand returned status {:?}", status.code())
            }
        }
        None => {
            // timeout
            child.kill()?;
            child.wait()?;
            fail!(HookError, "subcommand timed out after {:?}", timeout)
        }
    }
}

/// JSON output from the hook command (parsed with serde)
#[derive(Debug, Deserialize)]
pub struct Output {
    /// Latest block height
    pub latest_block_height: block::Height,
}

#[cfg(test)]
mod tests {
    use crate::config::chain::HookConfig;

    #[test]
    fn hook_test() {
        // TODO(tarcieri): write real tests for the hook subsystem
        let _ = super::run(&HookConfig {
            cmd: ["todo", "real", "example"]
                .iter()
                .map(|str| str.into())
                .collect(),
            timeout_secs: Some(0),
            fail_closed: true,
        });
    }
}
