use crate::components::{contracts, io::*, scheduler::*, verifier::*};
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

    // FIXME: This should probably be extracted somewhere else,
    //        or just left up to the users of the module.
    pub fn run(&mut self) -> Result<Never, Error> {
        loop {
            if let Err(e) = self.verify_to_highest() {
                eprintln!("verification error: {}", e);
                color_backtrace::print_backtrace(
                    e.backtrace().unwrap(),
                    &mut color_backtrace::Settings::new(),
                )
                .unwrap();
            }

            if let Err(e) = self.detect_forks() {
                eprintln!("fork detection error: {}", e);
                color_backtrace::print_backtrace(
                    e.backtrace().unwrap(),
                    &mut color_backtrace::Settings::new(),
                )
                .unwrap();
            }

            // FIXME: Debug only, should be left up to users of the module.
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    pub fn is_trusted(&self, light_block: &LightBlock) -> bool {
        let in_store = self.state.trusted_store_reader.get(light_block.height());
        in_store.as_ref() == Some(light_block)
    }

    pub fn verify_to_highest(&mut self) -> Result<(), Error> {
        if self.state.trusted_store_reader.highest().is_none() {
            bail!(ErrorKind::NoInitialTrustedState)
        };

        let primary = self.state.peers.primary.clone();
        let target_block = match self.io.fetch_light_block(primary, LATEST_HEIGHT) {
            Ok(last_block) => last_block,
            Err(io_error) => bail!(ErrorKind::Io(io_error)),
        };

        self.verify_to_target(target_block.height())
    }

    pub fn verify_to_target(&mut self, target_height: Height) -> Result<(), Error> {
        if self.state.trusted_store_reader.highest().is_none() {
            bail!(ErrorKind::NoInitialTrustedState)
        };

        let options = self.options.with_now(self.clock.now());

        let mut current_height = target_height;

        for trusted_state in self.state.trusted_store_reader.highest_iter() {
            // dbg!(target_height, current_height, trusted_state.height());

            self.state.trace_block(target_height, current_height);

            if target_height <= trusted_state.height() {
                return Ok(());
            }

            let current_block = self.state.trusted_store_reader.get(current_height);
            let current_block = match current_block {
                Some(current_block) => current_block,
                None => {
                    let primary = self.state.peers.primary.clone();
                    match self.io.fetch_light_block(primary, current_height) {
                        Ok(current_block) => {
                            self.state.untrusted_store_writer.add(current_block.clone());
                            current_block
                        }
                        Err(e) => bail!(ErrorKind::Io(e)),
                    }
                }
            };

            let verdict = self.verify_light_block(&current_block, &trusted_state, &options);
            // dbg!(&verdict);

            match verdict {
                Verdict::Success => {
                    // TODO: Refactor as a single method call
                    self.state.untrusted_store_writer.remove(&current_block);
                    self.state.trusted_store_writer.add(current_block.clone());
                }
                Verdict::Invalid(e) => {
                    // TODO: Refactor as a single method call
                    self.state.trusted_store_writer.remove(&current_block);
                    self.state.untrusted_store_writer.add(current_block.clone());

                    bail!(ErrorKind::InvalidLightBlock(e))
                }
                Verdict::NotEnoughTrust(_) => {
                    // TODO: Refactor as a single method call
                    self.state.trusted_store_writer.remove(&current_block);
                    self.state.untrusted_store_writer.add(current_block.clone());
                }
            }

            let scheduled_height = self.scheduler.schedule(
                &self.state.trusted_store_reader,
                current_height,
                target_height,
            );

            // dbg!(scheduled_height);

            postcondition!(contracts::schedule::postcondition(
                target_height,
                scheduled_height,
                &self.state.trusted_store_reader,
                &self.state.untrusted_store_reader
            ));

            current_height = scheduled_height;
        }

        postcondition!(
            contracts::verify::trusted_store_contains_block_at_target_height(
                target_height,
                &self.state.trusted_store_reader,
            )
        );

        Ok(())
    }

    pub fn get_trace(&self, target_height: Height) -> Vec<LightBlock> {
        self.state.get_trace(target_height)
    }

    pub fn verify_light_block(
        &self,
        light_block: &LightBlock,
        trusted_state: &TrustedState,
        options: &VerificationOptions,
    ) -> Verdict {
        self.verifier
            .validate_light_block(light_block, trusted_state, options)
            .and_then(|| {
                self.verifier
                    .verify_overlap(light_block, trusted_state, options)
            })
            .and_then(|| {
                self.verifier
                    .has_sufficient_voting_power(light_block, options)
            })
    }

    pub fn detect_forks(&self) -> Result<(), Error> {
        let light_blocks = self.state.trusted_store_reader.all();
        let result = self.fork_detector.detect(light_blocks);

        match result {
            ForkDetection::NotDetected => (),    // TODO
            ForkDetection::Detected(_, _) => (), // TODO
        }

        Ok(())
    }
}
