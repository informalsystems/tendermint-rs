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

        let target_block = match self.fetch_light_block(LATEST_HEIGHT) {
            Ok(last_block) => last_block,
            Err(_) => return, // No block to sync up to, abort. TODO: Deal with error
        };

        self.verify_loop(trusted_state, target_block);
    }

    fn verify_loop(&mut self, trusted_state: LightBlock, target_block: LightBlock) {
        let target_height = target_block.height;

        precondition!(
            contracts::verify::trusted_state_contains_block_within_trusting_period(
                &self.state.trusted_store_reader,
                self.options.trusting_period,
                self.options.now
            )
        );

        precondition!(
            contracts::verify::target_height_greater_than_all_blocks_in_trusted_store(
                target_height,
                &self.state.trusted_store_reader,
            )
        );

        let mut light_block = target_block;

        loop {
            let verif_result = self.verify_light_block(&light_block, &trusted_state, &self.options);

            if let VerifierOutput::Success = verif_result {
                self.state.add_trusted_state(light_block.clone());
            }

            let schedule = self.schedule(&light_block, &trusted_state, verif_result);

            match schedule {
                SchedulerOutput::Done => break,
                SchedulerOutput::Abort => return,
                SchedulerOutput::NextHeight(next_height) => {
                    light_block = match self.fetch_light_block(next_height) {
                        Ok(light_block) => light_block,
                        Err(_) => return, // couldn't fetch next block, abort.
                    }
                }
            }
        }

        postcondition!(
            contracts::verify::trusted_store_contains_block_at_target_height(
                target_height,
                &self.state.trusted_store_reader,
            )
        );
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

pub mod contracts {
    pub mod verify {
        use crate::prelude::*;

        pub fn trusted_state_contains_block_within_trusting_period(
            trusted_store: &StoreReader<Trusted>,
            trusting_period: Duration,
            now: SystemTime,
        ) -> bool {
            trusted_store
                .all()
                .iter()
                .any(|lb| is_within_trust_period(lb, trusting_period, now))
        }

        pub fn target_height_greater_than_all_blocks_in_trusted_store(
            target_height: Height,
            trusted_store: &StoreReader<Trusted>,
        ) -> bool {
            trusted_store
                .all()
                .iter()
                .all(|lb| lb.height < target_height)
        }

        pub fn trusted_store_contains_block_at_target_height(
            target_height: Height,
            trusted_store: &StoreReader<Trusted>,
        ) -> bool {
            trusted_store.get(target_height).is_some()
        }

        fn is_within_trust_period(
            light_block: &LightBlock,
            trusting_period: Duration,
            now: SystemTime,
        ) -> bool {
            let header_time = light_block.header().bft_time;
            let expires_at = header_time + trusting_period;

            header_time < now && expires_at > now && header_time <= now
        }
    }
}
