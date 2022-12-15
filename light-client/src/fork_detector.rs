//! Fork detection data structures and implementation.

use core::marker::PhantomData;

use tendermint::{crypto::Sha256, merkle::MerkleHash};

use crate::{
    errors::{Error, ErrorDetail},
    state::State,
    store::memory::MemoryStore,
    supervisor::Instance,
    verifier::{
        errors::ErrorExt,
        types::{LightBlock, PeerId, Status},
    },
};

/// Result of fork detection
#[derive(Debug)]
pub enum ForkDetection {
    /// One or more forks have been detected
    Detected(Vec<Fork>),
    /// No fork has been detected
    NotDetected,
}

/// Types of fork
#[derive(Debug)]
// To be fixed in 0.24
#[allow(clippy::large_enum_variant)]
pub enum Fork {
    /// An actual fork was found for this `LightBlock`
    Forked {
        /// Light block fetched from the primary
        primary: LightBlock,
        /// Light block fetched from a witness
        witness: LightBlock,
    },
    /// The node has been deemed faulty for this `LightBlock`
    Faulty(LightBlock, ErrorDetail),
    /// The node has timed out
    Timeout(PeerId, ErrorDetail),
}

/// Interface for a fork detector
pub trait ForkDetector: Send + Sync {
    /// Detect forks using the given verified block, trusted block,
    /// and list of witnesses to verify the given light block against.
    fn detect_forks(
        &self,
        verified_block: &LightBlock,
        trusted_block: &LightBlock,
        witnesses: Vec<&Instance>,
    ) -> Result<ForkDetection, Error>;
}

/// A production-ready fork detector which compares
/// light blocks fetched from the witnesses by hash.
/// If the hashes don't match, this fork detector
/// then attempts to verify the light block pulled from
/// the witness against a light block containing only
/// the given trusted state, and then:
///
/// - If the verification succeeds, we have a real fork
/// - If verification fails because of lack of trust, we have a potential fork.
/// - If verification fails for any other reason, the witness is deemed faulty.
pub struct ProvidedForkDetector<H> {
    _crypto: PhantomData<H>,
}

pub type ProdForkDetector = ProvidedForkDetector<Sha256>;

impl<H> ProvidedForkDetector<H> {
    /// Construct a new fork detector that will use the given header hasher.
    pub fn new() -> Self {
        Self {
            _crypto: PhantomData,
        }
    }
}

impl<H: Default> Default for ProvidedForkDetector<H> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H> ForkDetector for ProvidedForkDetector<H>
where
    // Sync + Send have to be added only because of
    // forbid(unsafe_code)
    H: MerkleHash + Default + Sync + Send,
{
    /// Perform fork detection. See the documentation `ProdForkDetector` for details.
    fn detect_forks(
        &self,
        verified_block: &LightBlock,
        trusted_block: &LightBlock,
        witnesses: Vec<&Instance>,
    ) -> Result<ForkDetection, Error> {
        let primary_hash = verified_block.signed_header.header.hash_with::<H>();

        let mut forks = Vec::with_capacity(witnesses.len());

        for witness in witnesses {
            let mut state = State::new(MemoryStore::new());

            let (witness_block, _) = witness
                .light_client
                .get_or_fetch_block(verified_block.height(), &mut state)?;

            let witness_hash = witness_block.signed_header.header.hash_with::<H>();

            if primary_hash == witness_hash {
                // Hashes match, continue with next witness, if any.
                continue;
            }

            state
                .light_store
                .insert(trusted_block.clone(), Status::Verified);

            state
                .light_store
                .insert(witness_block.clone(), Status::Unverified);

            let result = witness
                .light_client
                .verify_to_target(verified_block.height(), &mut state);

            match result {
                Ok(_) => forks.push(Fork::Forked {
                    primary: verified_block.clone(),
                    witness: witness_block,
                }),
                Err(Error(e, _)) if e.has_expired() => {
                    forks.push(Fork::Forked {
                        primary: verified_block.clone(),
                        witness: witness_block,
                    });
                },
                Err(Error(e, _)) => {
                    if e.is_timeout().is_some() {
                        forks.push(Fork::Timeout(witness_block.provider, e))
                    } else {
                        forks.push(Fork::Faulty(witness_block, e))
                    }
                },
            }
        }

        if forks.is_empty() {
            Ok(ForkDetection::NotDetected)
        } else {
            Ok(ForkDetection::Detected(forks))
        }
    }
}
