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

    pub fn verify_to_highest(&mut self) -> Result<(), Error> {
        if self
            .state
            .light_store
            .latest(VerifiedStatus::Verified)
            .is_none()
        {
            bail!(ErrorKind::NoInitialTrustedState)
        };

        let primary = self.state.peers.primary.clone();
        let target_block = match self.io.fetch_light_block(primary, LATEST_HEIGHT) {
            Ok(last_block) => last_block,
            Err(io_error) => bail!(ErrorKind::Io(io_error)),
        };

        self.verify_to_target(target_block.height())
    }

    #[post(
        ret.is_ok() ==> contracts::trusted_store_contains_block_at_target_height(
            self.state.light_store.as_ref(),
            target_height,
        )
    )]
    pub fn verify_to_target(&mut self, target_height: Height) -> Result<(), Error> {
        // TODO: Should this be a precondition?
        if self
            .state
            .light_store
            .latest(VerifiedStatus::Verified)
            .is_none()
        {
            bail!(ErrorKind::NoInitialTrustedState)
        };

        let options = self.options.with_now(self.clock.now());

        let mut current_height = target_height;

        // TODO: Add invariant and measure
        loop {
            let trusted_state = self
                .state
                .light_store
                .latest(VerifiedStatus::Verified)
                .unwrap(); // SAFETY: Checked above

            // dbg!(target_height, current_height, trusted_state.height());

            self.state.trace_block(target_height, current_height);

            if target_height <= trusted_state.height() {
                return Ok(());
            }

            let current_block = self
                .state
                .light_store
                .get(current_height, VerifiedStatus::Verified)
                .or_else(|| {
                    self.state
                        .light_store
                        .get(current_height, VerifiedStatus::Unverified)
                });

            let current_block = match current_block {
                Some(current_block) => current_block,
                None => {
                    let primary = self.state.peers.primary.clone();
                    match self.io.fetch_light_block(primary, current_height) {
                        Ok(current_block) => {
                            self.state
                                .light_store
                                .insert(current_block.clone(), VerifiedStatus::Unverified);

                            current_block
                        }
                        Err(e) => bail!(ErrorKind::Io(e)),
                    }
                }
            };

            let verdict = self
                .verifier
                .verify(&current_block, &trusted_state, &options);
            // dbg!(&verdict);

            match verdict {
                Verdict::Success => {
                    self.state
                        .light_store
                        .insert(current_block, VerifiedStatus::Verified);
                }
                Verdict::Invalid(e) => {
                    self.state
                        .light_store
                        .insert(current_block, VerifiedStatus::Failed);
                    bail!(ErrorKind::InvalidLightBlock(e))
                }
                Verdict::NotEnoughTrust(_) => {
                    self.state
                        .light_store
                        .insert(current_block, VerifiedStatus::Unverified);
                }
            }

            let scheduled_height = self.scheduler.schedule(
                self.state.light_store.as_ref(),
                current_height,
                target_height,
            );

            // dbg!(scheduled_height);

            current_height = scheduled_height;
        }
    }

    pub fn get_trace(&self, target_height: Height) -> Vec<LightBlock> {
        self.state.get_trace(target_height)
    }

    pub fn detect_forks(&self) -> Result<(), Error> {
        let light_blocks = self.state.light_store.all(VerifiedStatus::Verified);
        let result = self.fork_detector.detect(light_blocks);

        match result {
            ForkDetection::NotDetected => (),    // TODO
            ForkDetection::Detected(_, _) => (), // TODO
        }

        Ok(())
    }
}
