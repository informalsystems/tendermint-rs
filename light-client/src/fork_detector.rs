use serde::{Deserialize, Serialize};

use crate::{
    errors::{Error, ErrorExt, ErrorKind},
    operations::{HeaderHasher, ProdHeaderHasher},
    state::State,
    store::{memory::MemoryStore, VerifiedStatus},
    supervisor::Instance,
    types::LightBlock,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ForkDetection {
    Detected(Vec<Fork>),
    NotDetected,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Fork {
    Forked(LightBlock),
    Faulty(LightBlock, ErrorKind),
}

pub trait ForkDetector: Send {
    fn detect_forks(
        &self,
        light_block: &LightBlock,
        trusted_state: &LightBlock,
        witnesses: Vec<&Instance>,
    ) -> Result<ForkDetection, Error>;
}

pub struct ProdForkDetector {
    header_hasher: Box<dyn HeaderHasher>,
}

impl ProdForkDetector {
    pub fn new(header_hasher: impl HeaderHasher + 'static) -> Self {
        Self {
            header_hasher: Box::new(header_hasher),
        }
    }
}

impl Default for ProdForkDetector {
    fn default() -> Self {
        Self::new(ProdHeaderHasher)
    }
}

impl ForkDetector for ProdForkDetector {
    fn detect_forks(
        &self,
        light_block: &LightBlock,
        trusted_state: &LightBlock,
        witnesses: Vec<&Instance>,
    ) -> Result<ForkDetection, Error> {
        let primary_hash = self.header_hasher.hash(&light_block.signed_header.header);

        let mut forks = Vec::with_capacity(witnesses.len());

        for witness in witnesses {
            let mut state = State::new(MemoryStore::new());

            let witness_block = witness
                .light_client
                .get_or_fetch_block(light_block.height(), &mut state)
                .unwrap(); // FIXME: unwrap

            let witness_hash = self.header_hasher.hash(&witness_block.signed_header.header);

            if primary_hash == witness_hash {
                // Hashes match, continue with next witness, if any.
                continue;
            }

            state
                .light_store
                .update(trusted_state.clone(), VerifiedStatus::Verified);

            state
                .light_store
                .update(witness_block.clone(), VerifiedStatus::Unverified);

            let result = witness
                .light_client
                .verify_to_target(light_block.height(), &mut state);

            match result {
                Ok(_) => forks.push(Fork::Forked(witness_block)),
                Err(e) if e.kind().has_expired() => forks.push(Fork::Forked(witness_block)),
                Err(e) => forks.push(Fork::Faulty(witness_block, e.kind().clone())),
            }
        }

        if forks.is_empty() {
            Ok(ForkDetection::NotDetected)
        } else {
            Ok(ForkDetection::Detected(forks))
        }
    }
}
