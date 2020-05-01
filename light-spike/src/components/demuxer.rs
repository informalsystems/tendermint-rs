use super::{io::*, scheduler::*, verifier::*};
use crate::prelude::*;

pub struct Demuxer {
    state: State,
    options: VerificationOptions,
    scheduler: Box<dyn Scheduler>,
    verifier: Box<dyn Verifier>,
    fork_detector: Box<dyn ForkDetector>,
    io: Box<dyn Io>,
}

impl Demuxer {
    pub fn new(
        state: State,
        options: VerificationOptions,
        scheduler: impl Scheduler + 'static,
        verifier: impl Verifier + 'static,
        fork_detector: impl ForkDetector + 'static,
        io: impl Io + 'static,
    ) -> Self {
        Self {
            state,
            options,
            scheduler: Box::new(scheduler),
            verifier: Box::new(verifier),
            fork_detector: Box::new(fork_detector),
            io: Box::new(io),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.verify();
            self.detect_forks();
        }
    }

    pub fn verify(&mut self) {
        let trusted_state = match self.state.trusted_store_reader.latest() {
            Some(trusted_state) => trusted_state,
            None => return, // No trusted state to start from, abort.
        };

        let last_block = match self.fetch_light_block(LATEST_HEIGHT) {
            Ok(last_block) => last_block,
            Err(_) => return, // No block to sync up to, abort. TODO: Deal with error
        };

        let mut light_block = last_block;

        loop {
            let verif_result = self.verify_light_block(&light_block, &trusted_state, &self.options);

            if let VerifierOutput::Success = verif_result {
                self.state.add_trusted_state(light_block.clone());
            }

            let schedule = self.schedule(&light_block, &trusted_state, verif_result);

            match schedule {
                SchedulerOutput::Done => (),
                SchedulerOutput::Abort => (),
                SchedulerOutput::NextHeight(next_height) => {
                    light_block = match self.fetch_light_block(next_height) {
                        Ok(light_block) => light_block,
                        Err(_) => return, // couldn't fetch next block, abort.
                    }
                }
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

    pub fn detect_forks(&self) {
        let light_blocks = self.state.trusted_store_reader.all();
        let input = ForkDetectorInput::Detect(light_blocks);

        let result = self.fork_detector.process(input);

        match result {
            ForkDetectorOutput::NotDetected => (),    // TODO
            ForkDetectorOutput::Detected(_, _) => (), // TODO
        }
    }

    pub fn fetch_light_block(&self, height: Height) -> Result<LightBlock, IoError> {
        let input = IoInput::FetchLightBlock(height);
        let result = self.io.process(input)?;

        match result {
            IoOutput::FetchedLightBlock(light_block) => Ok(light_block),
        }
    }
}
