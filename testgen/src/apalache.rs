use crate::{command::*, tester::TestEnv};
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

pub enum ApalacheRun {
    /// Apalache has found a counterexample
    Counterexample(CommandRun),
    /// Apalache has not found a counterexample up to specified length bound
    NoCounterexample(CommandRun),
    /// Apalache has found a deadlock
    Deadlock(CommandRun),
    /// Apalache model checking run failed (e.g. a parsing error)
    ModelError(CommandRun),
    /// Apalache returned an unknown error code
    Unknown(CommandRun),
    /// The tool has reached the specified timeout without producing an answer
    Timeout(CommandRun),
}

impl ApalacheRun {
    pub fn message(&self) -> &str {
        match self {
            ApalacheRun::Counterexample(_) => "Apalache has generated a counterexample",
            ApalacheRun::NoCounterexample(_) => "Apalache failed to generate a counterexample; consider increasing the length bound, or changing your test",
            ApalacheRun::Deadlock(_) => "Apalache has found a deadlock; please inspect your model and test",
            ApalacheRun::ModelError(_) => "Apalache failed to process the model; please check it",
            ApalacheRun::Unknown(_) => "Apalache has generated an unknown outcome; please contact Apalache developers",
            ApalacheRun::Timeout(_) => "Apalache failed to generate a counterexample within given time; consider increasing the timeout, or changing your test",
        }
    }
}

pub fn run_apalache_test(dir: &str, test: ApalacheTestCase) -> io::Result<ApalacheRun> {
    let inv = format!("{}Inv", test.test);

    // Mutate the model: negate the test assertion to get the invariant to check
    let mutation_failed = || {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "failed to mutate the model and add invariant",
        )
    };
    let env = TestEnv::new(dir).ok_or_else(mutation_failed)?;
    let model = env.read_file(&test.model).unwrap();
    let mut new_model = String::new();
    for line in model.lines() {
        if line.starts_with(&inv) {
            // invariant already present; skip mutation
            new_model.clear();
            break;
        }
        if line.starts_with("======") {
            new_model += &format!("{} == ~{}\n", inv, test.test);
        }
        new_model += line;
        new_model += "\n";
    }
    if !new_model.is_empty() {
        env.write_file(&test.model, &new_model)
            .ok_or_else(mutation_failed)?;
    }

    // Run Apalache, and process the result
    let mut cmd = Command::new();
    if let Some(timeout) = test.timeout {
        cmd.program("timeout");
        cmd.arg(&timeout.to_string());
        cmd.arg("apalache-mc");
    } else {
        cmd.program("apalache-mc");
    }
    cmd.arg("check");
    cmd.arg_from_parts(vec!["--inv=", &inv]);
    cmd.arg("--init=InitTest");
    cmd.arg("--next=NextTest");
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
                    Ok(ApalacheRun::NoCounterexample(run))
                } else if run.stdout.contains("The outcome is: Error") {
                    Ok(ApalacheRun::Counterexample(run))
                } else if run.stdout.contains("The outcome is: Deadlock") {
                    Ok(ApalacheRun::Deadlock(run))
                } else {
                    Ok(ApalacheRun::Unknown(run))
                }
            } else if let Some(code) = run.status.code() {
                match code {
                    99 => Ok(ApalacheRun::ModelError(run)),
                    124 => Ok(ApalacheRun::Timeout(run)),
                    _ => Ok(ApalacheRun::Unknown(run)),
                }
            } else {
                Ok(ApalacheRun::Timeout(run))
            }
        }
        Err(e) => Err(e),
    }
}
