use crate::command::*;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Deserialize, Clone, Debug)]
pub struct ApalacheTestBatch {
    pub description: String,
    pub model: String,
    pub length: Option<u64>,
    pub timeout: Option<u64>,
    pub tests: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApalacheTestCase {
    pub model: String,
    pub test: String,
    pub length: Option<u64>,
    pub timeout: Option<u64>,
}

pub enum ApalacheResult {
    /// Apalache has not found an error up to specified length bound
    NoError(CommandRun),
    /// Apalache has found an error
    Error(CommandRun),
    /// Apalache has found a deadlock
    Deadlock(CommandRun),
    /// Apalache model checking run failed (e.g. a parsing error)
    ModelError(CommandRun),
    /// Apalache returned an unknown error code
    Unknown(CommandRun),
    /// The tool has reached the specified timeout without producing an answer
    Timeout(CommandRun),
    /// Failed to execute the tool
    Failure(io::Error),
}

pub fn run_apalache_test(dir: &str, test: ApalacheTestCase) -> ApalacheResult {
    let mut cmd = Command::new();
    if let Some(timeout) = test.timeout {
        cmd.program("timeout");
        cmd.arg(&timeout.to_string());
        cmd.arg("apalache-mc");
    } else {
        cmd.program("apalache-mc");
    }
    cmd.arg("check");
    cmd.arg_from_parts(vec!["--inv=", &test.test]);
    if let Some(length) = test.length {
        cmd.arg_from_parts(vec!["--length=", &length.to_string()]);
    }
    cmd.arg(&test.model);
    if !dir.is_empty() {
        cmd.current_dir(dir);
    }
    match cmd.spawn() {
        Ok(run) => {
            if run.status.success() {
                if run.stdout.contains("The outcome is: NoError") {
                    ApalacheResult::NoError(run)
                } else if run.stdout.contains("The outcome is: Error") {
                    ApalacheResult::Error(run)
                } else if run.stdout.contains("The outcome is: Deadlock") {
                    ApalacheResult::Deadlock(run)
                } else {
                    ApalacheResult::Unknown(run)
                }
            } else if let Some(code) = run.status.code() {
                match code {
                    99 => ApalacheResult::ModelError(run),
                    124 => ApalacheResult::Timeout(run),
                    _ => ApalacheResult::Unknown(run),
                }
            } else {
                ApalacheResult::Timeout(run)
            }
        }
        Err(e) => ApalacheResult::Failure(e),
    }
}
