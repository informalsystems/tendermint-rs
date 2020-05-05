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
        checked_header: LightBlock,
        trusted_state: TrustedState,
        verifier_result: VerifierOutput,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SchedulerOutput {
    Done,
    InvalidLightBlock(VerificationError),
    NextHeight(Height),
}

pub fn schedule(input: SchedulerInput) -> SchedulerOutput {
    match input {
        SchedulerInput::Schedule {
            checked_header,
            trusted_state,
            verifier_result,
        } => match verifier_result {
            VerifierOutput::Success => SchedulerOutput::Done,
            VerifierOutput::Invalid(e) => SchedulerOutput::InvalidLightBlock(e),
            VerifierOutput::NotEnoughTrust => {
                let pivot_height = compute_pivot_height(&checked_header, &trusted_state);
                SchedulerOutput::NextHeight(pivot_height)
            }
        },
    }
}

fn compute_pivot_height(checked_header: &LightBlock, trusted_state: &TrustedState) -> Height {
    let trusted_height = trusted_state.height;
    let untrusted_height = checked_header.height;

    assert!(trusted_height < untrusted_height);

    // Equivalent to (trusted_height + untrusted_height) / 2, but avoid overflows
    trusted_height + (untrusted_height - trusted_height) / 2
}
