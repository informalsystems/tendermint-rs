use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{io::*, scheduler::*, verifier::*};
use crate::prelude::*;

#[derive(Clone, Debug, Error, PartialEq, Serialize, Deserialize)]
pub enum DemuxerError {
    #[error("scheduler error")]
    Scheduler(SchedulerError),
    #[error("verifier error")]
    Verifier(VerifierError),
    #[error("fork detector")]
    ForkDetector(ForkDetectorError),
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

    pub fn verify_height(
        &self,
        height: Height,
        trusted_state: TrustedState,
        options: VerificationOptions,
    ) -> Result<Vec<LightBlock>, DemuxerError> {
        let input = SchedulerInput::VerifyHeight {
            height,
            trusted_state,
            options,
        };

        let result = self.run_scheduler(input)?;

        match result {
            SchedulerOutput::TrustedStates(trusted_states) => {
                // self.state.add_trusted_states(trusted_states.clone());
                Ok(trusted_states)
            }
        }
    }

    pub fn verify_light_block(
        &self,
        light_block: LightBlock,
        trusted_state: TrustedState,
        options: VerificationOptions,
    ) -> Result<Vec<LightBlock>, DemuxerError> {
        let input = SchedulerInput::VerifyLightBlock {
            light_block,
            trusted_state,
            options,
        };

        let result = self.run_scheduler(input)?;

        match result {
            SchedulerOutput::TrustedStates(trusted_states) => {
                // self.state.add_trusted_states(trusted_states.clone());
                Ok(trusted_states)
            }
        }
    }

    pub fn validate_light_block(
        &self,
        light_block: LightBlock,
        trusted_state: TrustedState,
        options: VerificationOptions,
    ) -> Result<LightBlock, DemuxerError> {
        let input = VerifierInput::VerifyLightBlock {
            light_block,
            trusted_state,
            options,
        };

        let result = self
            .verifier
            .process(input)
            .map_err(|e| DemuxerError::Verifier(e))?;

        match result {
            VerifierOutput::ValidLightBlock(valid_light_block) => {
                // self.state.add_valid_light_block(valid_light_block.clone());
                Ok(valid_light_block)
            }
        }
    }

    pub fn detect_forks(&self) -> Result<(), DemuxerError> {
        let light_blocks = self.state.trusted_store_reader.all();
        let input = ForkDetectorInput::Detect(light_blocks);

        let result = self
            .fork_detector
            .process(input)
            .map_err(DemuxerError::ForkDetector)?;

        match result {
            ForkDetectorOutput::NotDetected => Ok(()),
        }
    }

    pub fn fetch_light_block(&self, height: Height) -> Result<LightBlock, DemuxerError> {
        let input = IoInput::FetchLightBlock(height);

        let result = self.io.process(input).map_err(|e| DemuxerError::Io(e))?;

        match result {
            IoOutput::FetchedLightBlock(lb) => {
                // self.state.add_fetched_light_block(lb.clone());
                Ok(lb)
            }
        }
    }

    fn handle_request(&self, request: SchedulerRequest) -> Result<SchedulerResponse, DemuxerError> {
        match request {
            SchedulerRequest::GetLightBlock(height) => self
                .fetch_light_block(height)
                .map(|lb| SchedulerResponse::LightBlock(lb)),

            SchedulerRequest::VerifyLightBlock {
                light_block,
                trusted_state,
                options,
            } => match self.verify_light_block(light_block, trusted_state, options) {
                Ok(ts) => Ok(SchedulerResponse::Verified(Ok(ts))),
                Err(DemuxerError::Verifier(err)) => Ok(SchedulerResponse::Verified(Err(err))),
                Err(err) => Err(err),
            },

            SchedulerRequest::ValidateLightBlock {
                light_block,
                trusted_state,
                options,
            } => match self.validate_light_block(light_block, trusted_state, options) {
                Ok(ts) => Ok(SchedulerResponse::Validated(Ok(ts))),
                Err(DemuxerError::Verifier(err)) => Ok(SchedulerResponse::Validated(Err(err))),
                Err(err) => Err(err),
            },
        }
    }

    fn run_scheduler(&self, input: SchedulerInput) -> Result<SchedulerOutput, DemuxerError> {
        let scheduler = Gen::new(|co| {
            self.scheduler
                .process(self.state.trusted_store_reader(), input, co)
        });

        let result = drain(scheduler, SchedulerResponse::Init, |req| {
            self.handle_request(req)
        })?;

        result.map_err(|e| DemuxerError::Scheduler(e))
    }
}
