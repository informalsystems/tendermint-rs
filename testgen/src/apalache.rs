use crate::command::*;
use serde::Deserialize;
use std::io;

#[derive(Deserialize, Clone, Debug)]
pub struct ApalacheTestBatch {
    pub description: String,
    pub model: String,
    pub length: Option<u64>,
    pub timeout: Option<u64>,
    pub tests: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ApalacheTestCase {
    pub model: String,
    pub test: String,
    pub length: Option<u64>,
    pub timeout: Option<u64>,
}

pub fn run_apalache_test(dir: &str, test: ApalacheTestCase) -> io::Result<CommandRun> {
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
                Ok(run)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    run.stdout.to_string(),
                ))
            }
        }
        Err(e) => Err(e),
    }
}
