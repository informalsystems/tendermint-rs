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
            if let Err(e) = self.sync_to_highest() {
                eprintln!("verification error: {}", e);
                color_backtrace::print_backtrace(
                    e.backtrace().unwrap(),
                    &mut color_backtrace::Settings::new(),
                )
                .unwrap();
            }

            dbg!(&self.state.trusted_store_reader.highest_height());

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

    fn is_trusted(&self, light_block: &LightBlock) -> bool {
        let in_store = self.state.trusted_store_reader.get(light_block.height);
        in_store.as_ref() == Some(light_block)
    }

    pub fn sync_to_highest(&mut self) -> Result<(), Error> {
        if self.state.trusted_store_reader.highest().is_none() {
            bail!(ErrorKind::NoInitialTrustedState)
        };

        let primary = self.state.peers.primary.clone();
        let target_block = match self.fetch_light_block(primary, LATEST_HEIGHT) {
            Ok(last_block) => last_block,
            Err(io_error) => bail!(ErrorKind::Io(io_error)),
        };

        if !self.is_trusted(&target_block) {
            self.verify_to_target(target_block.height)?;
        }

        Ok(())
    }

    fn verify_to_target(&mut self, target_height: Height) -> Result<(), Error> {
        let options = self.options.with_now(self.clock.now());

        // TODO: Check this ahead of time
        precondition!(
            contracts::verify::trusted_state_contains_block_within_trusting_period(
                &self.state.trusted_store_reader,
                self.options.trusting_period,
                options.now
            )
        );

        // TODO: This might now be a good precondition if we need to verify
        //       intermediate blocks, eg. for the relayer.
        precondition!(
            contracts::verify::target_height_greater_than_all_blocks_in_trusted_store(
                target_height,
                &self.state.trusted_store_reader,
            )
        );

        let mut next_height = target_height;

        for trusted_state in self.state.trusted_store_reader.highest_iter() {
            if trusted_state.height >= target_height {
                break;
            }

            dbg!(target_height);
            dbg!(trusted_state.height);
            dbg!(next_height);

            let primary = self.state.peers.primary.clone();
            let current_block = match self.fetch_light_block(primary, next_height) {
                Ok(current_block) => current_block,
                Err(_) => return Ok(()),
            };

            let verdict = self.verify_light_block(&current_block, &trusted_state, &options);
            dbg!(&verdict);

            match verdict {
                Verdict::Success => {
                    self.state.trusted_store_writer.add(current_block);
                    continue;
                }
                Verdict::Invalid(e) => {
                    self.state.untrusted_store_writer.add(current_block);
                    bail!(ErrorKind::InvalidLightBlock(e))
                }
                Verdict::NotEnoughTrust => {
                    self.state.untrusted_store_writer.add(current_block.clone());
                    self.state.trace_block(target_height, current_block.height);

                    let scheduled_height = self.schedule(&current_block, &trusted_state);
                    dbg!(&scheduled_height);

                    if scheduled_height <= trusted_state.height {
                        bail!(ErrorKind::BisectionFailed(target_height, scheduled_height));
                    }

                    postcondition!(contracts::schedule::postcondition(
                        &trusted_state,
                        target_height,
                        scheduled_height,
                        &self.state.trusted_store_reader,
                        &self.state.untrusted_store_reader
                    ));

                    next_height = scheduled_height;
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
    ) -> Verdict {
        let verdict = self
            .verifier
            .validate_light_block(light_block, trusted_state, options);

        if let Verdict::Invalid(_) = verdict {
            return verdict;
        }

        let verdict = self
            .verifier
            .verify_overlap(light_block, trusted_state, options);

        if let Verdict::Invalid(_) = verdict {
            return verdict;
        }

        self.verifier
            .has_sufficient_voting_power(light_block, options)
    }

    pub fn schedule(&self, light_block: &LightBlock, trusted_state: &TrustedState) -> Height {
        self.scheduler.schedule(light_block, trusted_state)
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

    pub fn fetch_light_block(&mut self, peer: Peer, height: Height) -> Result<LightBlock, IoError> {
        self.io.fetch_light_block(peer, height)
    }
}
