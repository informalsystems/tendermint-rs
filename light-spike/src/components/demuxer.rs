use super::{contracts, io::*, scheduler::*, verifier::*};
use crate::prelude::*;

pub struct Demuxer {
    state: State,
    options: VerificationOptions,
    clock: Box<dyn Clock>,
    scheduler: Box<dyn Scheduler>,
    verifier: Box<dyn Verifier>,
    fork_detector: Box<dyn ForkDetector>,
    io: Box<dyn Io>,
}

impl Demuxer {
    pub fn new(
        state: State,
        options: VerificationOptions,
        clock: impl Clock + 'static,
        scheduler: impl Scheduler + 'static,
        verifier: impl Verifier + 'static,
        fork_detector: impl ForkDetector + 'static,
        io: impl Io + 'static,
    ) -> Self {
        Self {
            state,
            options,
            clock: Box::new(clock),
            scheduler: Box::new(scheduler),
            verifier: Box::new(verifier),
            fork_detector: Box::new(fork_detector),
            io: Box::new(io),
        }
    }

    pub fn run(&mut self) -> Result<Never, Error> {
        loop {
            if let Err(e) = self.verify() {
                eprintln!("verification error: {}", e);
                color_backtrace::print_backtrace(
                    e.backtrace().unwrap(),
                    &mut color_backtrace::Settings::new(),
                )
                .unwrap();
            }

            dbg!(&self.state.trusted_store_reader.latest_height());

            if let Err(e) = self.detect_forks() {
                eprintln!("fork detection error: {}", e);
                color_backtrace::print_backtrace(
                    e.backtrace().unwrap(),
                    &mut color_backtrace::Settings::new(),
                )
                .unwrap();
            }

            std::thread::sleep(Duration::from_secs(1));
        }
    }

    fn is_trusted(&self, light_block: &LightBlock) -> bool {
        let in_store = self.state.trusted_store_reader.get(light_block.height);
        in_store.as_ref() == Some(light_block)
    }

    pub fn verify(&mut self) -> Result<(), Error> {
        if self.state.trusted_store_reader.latest().is_none() {
            bail!(ErrorKind::NoInitialTrustedState)
        };

        let target_block = match self.fetch_light_block(LATEST_HEIGHT) {
            Ok(last_block) => last_block,
            Err(io_error) => bail!(ErrorKind::Io(io_error)),
        };

        if !self.is_trusted(&target_block) {
            self.verify_loop(target_block.height)?;
        }

        Ok(())
    }

    fn verify_loop(&mut self, target_height: Height) -> Result<(), Error> {
        let options = self.options.set_now(self.clock.now());

        precondition!(
            contracts::verify::trusted_state_contains_block_within_trusting_period(
                &self.state.trusted_store_reader,
                self.options.trusting_period,
                options.now
            )
        );

        precondition!(
            contracts::verify::target_height_greater_than_all_blocks_in_trusted_store(
                target_height,
                &self.state.trusted_store_reader,
            )
        );

        let mut next_height = target_height;
        let mut trusted_state = self.state.trusted_store_reader.latest().unwrap();

        while trusted_state.height < target_height {
            trusted_state = self.state.trusted_store_reader.latest().unwrap();

            dbg!(target_height);
            dbg!(trusted_state.height);
            dbg!(next_height);

            let current_block = match self.fetch_light_block(next_height) {
                Ok(current_block) => current_block,
                Err(_) => return Ok(()),
            };

            let verif_result = self.verify_light_block(&current_block, &trusted_state, &options);
            dbg!(&verif_result);

            if let VerifierOutput::Success = verif_result {
                self.state.trusted_store_writer.add(current_block.clone());
            } else {
                self.state.untrusted_store_writer.add(current_block.clone());
            }

            let schedule = self.schedule(&current_block, &trusted_state, verif_result);
            dbg!(&schedule);

            match schedule {
                SchedulerOutput::Done => continue,
                SchedulerOutput::InvalidLightBlock(e) => {
                    bail!(ErrorKind::InvalidLightBlock(e));
                }
                SchedulerOutput::NextHeight(height) if height <= trusted_state.height => {
                    bail!(ErrorKind::BisectionFailed(target_height, height));
                }
                SchedulerOutput::NextHeight(height) => {
                    postcondition!(contracts::schedule::postcondition(
                        &trusted_state,
                        target_height,
                        height,
                        &self.state.trusted_store_reader,
                        &self.state.untrusted_store_reader
                    ));

                    next_height = height;
                }
            }
        }

        postcondition!(
            contracts::verify::trusted_store_contains_block_at_target_height(
                target_height,
                &self.state.trusted_store_reader,
            )
        );

        Ok(())
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
        checked_header: &LightBlock,
        trusted_state: &TrustedState,
        verifier_result: VerifierOutput,
    ) -> SchedulerOutput {
        let input = SchedulerInput::Schedule {
            checked_header: checked_header.clone(),
            trusted_state: trusted_state.clone(),
            verifier_result,
        };

        self.scheduler.process(input)
    }

    pub fn detect_forks(&self) -> Result<(), Error> {
        let light_blocks = self.state.trusted_store_reader.all();
        let input = ForkDetectorInput::Detect(light_blocks);

        let result = self.fork_detector.process(input);

        match result {
            ForkDetectorOutput::NotDetected => (),    // TODO
            ForkDetectorOutput::Detected(_, _) => (), // TODO
        }

        Ok(())
    }

    pub fn fetch_light_block(&self, height: Height) -> Result<LightBlock, IoError> {
        let input = IoInput::FetchLightBlock(height);
        let result = self.io.process(input)?;

        match result {
            IoOutput::FetchedLightBlock(light_block) => Ok(light_block),
        }
    }
}

