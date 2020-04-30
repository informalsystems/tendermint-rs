use serde::{Deserialize, Serialize};

use crate::prelude::*;

pub trait Scheduler {
    fn process(&self, input: SchedulerInput) -> SchedulerOutput;
}

impl<F> Scheduler for F
where
    F: Fn(SchedulerInput) -> SchedulerOutput,
{
    fn process(&self, input: SchedulerInput) -> SchedulerOutput {
        self(input)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SchedulerInput {
    Schedule {
        light_block: LightBlock,
        trusted_state: TrustedState,
        verifier_result: VerifierOutput,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SchedulerOutput {
    Abort,
    Done,
    NextHeight(Height),
}

pub fn schedule(input: SchedulerInput) -> SchedulerOutput {
    match input {
        SchedulerInput::Schedule {
            light_block,
            trusted_state,
            verifier_result,
        } => match verifier_result {
            VerifierOutput::Success => SchedulerOutput::Done,
            VerifierOutput::NotEnoughTrust => {
                SchedulerOutput::NextHeight(compute_pivot_height(&light_block, &trusted_state))
            }
            VerifierOutput::Invalid(_) => SchedulerOutput::Abort,
        },
    }
}

fn compute_pivot_height(light_block: &LightBlock, trusted_state: &TrustedState) -> Height {
    let trusted_height = trusted_state.height;
    let untrusted_height = light_block.height;

    assert!(trusted_height < untrusted_height);

    // Equivalent to (trusted_height + untrusted_height) / 2, but avoid overflows
    trusted_height + (untrusted_height - trusted_height) / 2
}
