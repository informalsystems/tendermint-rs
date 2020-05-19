use derive_more::Display;
use serde::{Deserialize, Serialize};
use contracts::*;

use crate::components::{io::*, scheduler::*, verifier::*};
use crate::contracts::*;
use crate::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct Options {
    pub trust_threshold: TrustThreshold,
    pub trusting_period: Duration,
    pub now: Time,
}

impl Options {
    pub fn with_now(self, now: Time) -> Self {
        Self { now, ..self }
    }
}

pub struct LightClient {
    state: State,
    options: Options,
    clock: Box<dyn Clock>,
    scheduler: Box<dyn Scheduler>,
    verifier: Box<dyn Verifier>,
    fork_detector: Box<dyn ForkDetector>,
    io: Box<dyn Io>,
}

impl LightClient {
    pub fn new(
        state: State,
        options: Options,
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
        let latest_verified = self.state.light_store.latest(VerifiedStatus::Verified);
        if latest_verified.is_none() {
            bail!(ErrorKind::NoInitialTrustedState)
        };

        let peer = self.state.peers.primary;
        let target_block = match self.io.fetch_light_block(peer, LATEST_HEIGHT) {
            Ok(last_block) => last_block,
            Err(io_error) => bail!(ErrorKind::Io(io_error)),
        };

        self.verify_to_target(target_block.height())
    }

    #[post(
        ret.is_ok() ==> trusted_store_contains_block_at_target_height(
            self.state.light_store.as_ref(),
            target_height,
        )
    )]
    pub fn verify_to_target(&mut self, target_height: Height) -> Result<(), Error> {
        let latest_verified = self.state.light_store.latest(VerifiedStatus::Verified);
        if latest_verified.is_none() {
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

            self.state.trace_block(target_height, current_height);

            if target_height <= trusted_state.height() {
                return Ok(());
            }

            let current_block = self.get_or_fetch_block(current_height)?;

            let verdict = self
                .verifier
                .verify(&current_block, &trusted_state, &options);

            match verdict {
                Verdict::Success => {
                    self.state
                        .light_store
                        .update(current_block, VerifiedStatus::Verified);
                }
                Verdict::Invalid(e) => {
                    self.state
                        .light_store
                        .update(current_block, VerifiedStatus::Failed);

                    bail!(ErrorKind::InvalidLightBlock(e))
                }
                Verdict::NotEnoughTrust(_) => {
                    self.state
                        .light_store
                        .update(current_block, VerifiedStatus::Unverified);
                }
            }

            let scheduled_height = self.scheduler.schedule(
                self.state.light_store.as_ref(),
                current_height,
                target_height,
            );

            current_height = scheduled_height;
        }
    }

    fn get_or_fetch_block(&mut self, current_height: Height) -> Result<LightBlock, Error> {
        let current_block = self
            .state
            .light_store
            .get(current_height, VerifiedStatus::Verified)
            .or_else(|| {
                self.state
                    .light_store
                    .get(current_height, VerifiedStatus::Unverified)
            });

        if let Some(current_block) = current_block {
            return Ok(current_block);
        }

        let peer = self.state.peers.primary;
        self.io
            .fetch_light_block(peer, current_height)
            .map(|current_block| {
                self.state
                    .light_store
                    .insert(current_block.clone(), VerifiedStatus::Unverified);

                current_block
            })
            .map_err(|e| ErrorKind::Io(e).into())
    }

    pub fn get_trace(&self, target_height: Height) -> Vec<LightBlock> {
        self.state.get_trace(target_height)
    }

    pub fn detect_forks(&self) -> Result<(), Error> {
        let light_blocks = self
            .state
            .light_store
            .all(VerifiedStatus::Verified)
            .collect();

        let result = self.fork_detector.detect(light_blocks);

        match result {
            ForkDetection::NotDetected => (),    // TODO
            ForkDetection::Detected(_, _) => (), // TODO
        }

        Ok(())
    }
}
