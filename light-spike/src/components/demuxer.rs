use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{io::*, scheduler::*, verifier::*};
use crate::prelude::*;

#[derive(Clone, Debug, Error, PartialEq, Serialize, Deserialize)]
pub enum DemuxerError {
    #[error("I/O error")]
    Io(IoError),
}

pub struct Demuxer {
    state: State,
    scheduler: Box<dyn Scheduler>,
    verifier: Box<dyn Verifier>,
    fork_detector: Box<dyn ForkDetector>,
    io: Box<dyn Io>,
}

impl Demuxer {
    pub fn new(
        state: State,
        scheduler: impl Scheduler + 'static,
        verifier: impl Verifier + 'static,
        fork_detector: impl ForkDetector + 'static,
        io: impl Io + 'static,
    ) -> Self {
        Self {
            state,
            scheduler: Box::new(scheduler),
            verifier: Box::new(verifier),
            fork_detector: Box::new(fork_detector),
            io: Box::new(io),
        }
    }

    pub fn run(&mut self) {
        // self.verify();
        // self.detect_forks();
    }

    pub fn verify(&mut self, mut light_block: LightBlock, options: VerificationOptions) {
        loop {
            let trusted_state = self.state.trusted_store_reader.latest().unwrap(); // FIXME
            let verif_result = self.verify_light_block(&light_block, &trusted_state, &options);

            if let VerifierOutput::Success = verif_result {
                self.state.add_trusted_state(light_block.clone());
            }

            let schedule = self.schedule(&light_block, &trusted_state, verif_result);

            match schedule {
                SchedulerOutput::Done => {
                    // Done
                }
                SchedulerOutput::NextHeight(next_height) => {
                    light_block = self.fetch_light_block(next_height).unwrap() // FIXME
                }
                SchedulerOutput::Abort => todo!(),
            }
        }
    }

    pub fn verify_light_block(
        &self,
        light_block: &LightBlock,
        trusted_state: &TrustedState,
        options: &VerificationOptions,
    ) -> VerifierOutput {
        let input = VerifierInput::VerifyLightBlock {
            light_block: light_block.clone(),
            trusted_state: trusted_state.clone(),
            options: options.clone(),
        };

        self.verifier.process(input)
    }

    pub fn schedule(
        &self,
        light_block: &LightBlock,
        trusted_state: &TrustedState,
        verifier_result: VerifierOutput,
    ) -> SchedulerOutput {
        let input = SchedulerInput::Schedule {
            light_block: light_block.clone(),
            trusted_state: trusted_state.clone(),
            verifier_result,
        };

        self.scheduler.process(input)
    }

    pub fn detect_forks(&self) -> Result<(), DemuxerError> {
        let light_blocks = self.state.trusted_store_reader.all();
        let input = ForkDetectorInput::Detect(light_blocks);

        let result = self.fork_detector.process(input);

        match result {
            ForkDetectorOutput::NotDetected => Ok(()),
            ForkDetectorOutput::Detected(_, _) => Ok(()), // TODO
        }
    }

    pub fn fetch_light_block(&self, height: Height) -> Result<LightBlock, DemuxerError> {
        let input = IoInput::FetchLightBlock(height);
        let result = self.io.process(input).map_err(|e| DemuxerError::Io(e))?;

        match result {
            IoOutput::FetchedLightBlock(light_block) => Ok(light_block),
        }
    }
}
