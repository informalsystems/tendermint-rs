use crate::command::*;
use serde::Deserialize;
use std::io;

#[derive(Deserialize, Clone, Debug)]
pub struct JsonatrTransform {
    pub input: String,
    pub include: Vec<String>,
    pub output: String,
}

pub fn run_jsonatr_transform(dir: &str, transform: JsonatrTransform) -> io::Result<CommandRun> {
    let mut cmd = Command::new();
    cmd.program("jsonatr");
    cmd.arg("--in");
    cmd.arg(&transform.input);
    cmd.arg("--out");
    cmd.arg(&transform.output);
    for include in transform.include {
        cmd.arg("--use");
        cmd.arg(&include);
    }
    if !dir.is_empty() {
        cmd.current_dir(dir);
    }
    match cmd.spawn() {
        Ok(run) => {
            if run.status.success() {
                Ok(run)
            } else {
                Err(io::Error::new(io::ErrorKind::Interrupted, run.stderr))
            }
        }
        Err(e) => Err(e),
    }
}
