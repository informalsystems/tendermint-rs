//! Provides an interface and default implementation of the `Verifier` component

use crate::predicates as preds;
use crate::{
    errors::ErrorExt,
    light_client::Options,
    operations::{
        CommitValidator, Hasher, ProdCommitValidator, ProdHasher, ProdVotingPowerCalculator,
        VotingPowerCalculator,
    },
    types::{LightBlock, Time},
};
use preds::{errors::VerificationError, ProdPredicates, VerificationPredicates};
use serde::{Deserialize, Serialize};

/// Represents the result of the verification performed by the
/// verifier component.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Verdict {
    /// Verification succeeded, the block is valid.
    Success,
    /// The minimum voting power threshold is not reached,
    /// the block cannot be trusted yet.
    NotEnoughTrust(VerificationError),
    /// Verification failed, the block is invalid.
    Invalid(VerificationError),
}

impl From<Result<(), VerificationError>> for Verdict {
    fn from(result: Result<(), VerificationError>) -> Self {
        match result {
            Ok(()) => Self::Success,
            Err(e) if e.not_enough_trust() => Self::NotEnoughTrust(e),
            Err(e) => Self::Invalid(e),
        }
    }
}

/// The verifier checks:
///
/// a) whether a given untrusted light block is valid, and
/// b) whether a given untrusted light block should be trusted
///    based on a previously verified block.
///
/// ## Implements
/// - [TMBC-VAL-CONTAINS-CORR.1]
/// - [TMBC-VAL-COMMIT.1]
pub trait Verifier: Send + Sync {
    /// Perform the verification.
    fn verify(
        &self,
        untrusted: &LightBlock,
        trusted: &LightBlock,
        options: &Options,
        now: Time,
    ) -> Verdict;
}

/// Production implementation of the verifier.
///
/// For testing purposes, this implementation is parametrized by:
/// - A set of predicates used to validate a light block
/// - A voting power calculator
/// - A commit validator
/// - A header hasher
///
/// For regular use, one can construct a standard implementation with `ProdVerifier::default()`.
pub struct ProdVerifier {
    predicates: Box<dyn VerificationPredicates>,
    voting_power_calculator: Box<dyn VotingPowerCalculator>,
    commit_validator: Box<dyn CommitValidator>,
    hasher: Box<dyn Hasher>,
}

impl ProdVerifier {
    /// Constructs a new instance of this struct
    pub fn new(
        predicates: impl VerificationPredicates + 'static,
        voting_power_calculator: impl VotingPowerCalculator + 'static,
        commit_validator: impl CommitValidator + 'static,
        hasher: impl Hasher + 'static,
    ) -> Self {
        Self {
            predicates: Box::new(predicates),
            voting_power_calculator: Box::new(voting_power_calculator),
            commit_validator: Box::new(commit_validator),
            hasher: Box::new(hasher),
        }
    }
}

impl Default for ProdVerifier {
    fn default() -> Self {
        Self::new(
            ProdPredicates::default(),
            ProdVotingPowerCalculator::default(),
            ProdCommitValidator::default(),
            ProdHasher::default(),
        )
    }
}

impl Verifier for ProdVerifier {
    fn verify(
        &self,
        untrusted: &LightBlock,
        trusted: &LightBlock,
        options: &Options,
        now: Time,
    ) -> Verdict {
        preds::verify(
            &*self.predicates,
            &*self.voting_power_calculator,
            &*self.commit_validator,
            &*self.hasher,
            trusted,
            untrusted,
            options,
            now,
        )
        .into()
    }
}
