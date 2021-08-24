//! Provides an interface and default implementation of the `Verifier` component

use crate::operations::voting_power::VotingPowerTally;
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
use preds::{
    errors::{VerificationError, VerificationErrorDetail},
    ProdPredicates, VerificationPredicates,
};
use serde::{Deserialize, Serialize};

/// Represents the result of the verification performed by the
/// verifier component.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Verdict {
    /// Verification succeeded, the block is valid.
    Success,
    /// The minimum voting power threshold is not reached,
    /// the block cannot be trusted yet.
    NotEnoughTrust(VotingPowerTally),
    /// Verification failed, the block is invalid.
    Invalid(VerificationErrorDetail),
}

impl From<Result<(), VerificationError>> for Verdict {
    fn from(result: Result<(), VerificationError>) -> Self {
        match result {
            Ok(()) => Self::Success,
            Err(VerificationError(e, _)) => match e.not_enough_trust() {
                Some(tally) => Self::NotEnoughTrust(tally),
                _ => Self::Invalid(e),
            },
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

pub trait VerifierComponents {
    type VerificationPredicates: VerificationPredicates;
    type VotingPowerCalculator: VotingPowerCalculator;
    type CommitValidator: CommitValidator;
    type Hasher: Hasher;
}

pub struct ProdVerifierComponents;

impl VerifierComponents for ProdVerifierComponents {
    type VerificationPredicates = ProdPredicates;
    type VotingPowerCalculator = ProdVotingPowerCalculator;
    type CommitValidator = ProdCommitValidator;
    type Hasher = ProdHasher;
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
pub struct ProdVerifier<C: VerifierComponents> {
    predicates: C::VerificationPredicates,
    voting_power_calculator: C::VotingPowerCalculator,
    commit_validator: C::CommitValidator,
    hasher: C::Hasher,
}

impl<C: VerifierComponents> ProdVerifier<C> {
    /// Constructs a new instance of this struct
    pub fn new(
        predicates: C::VerificationPredicates,
        voting_power_calculator: C::VotingPowerCalculator,
        commit_validator: C::CommitValidator,
        hasher: C::Hasher,
    ) -> Self {
        Self {
            predicates,
            voting_power_calculator,
            commit_validator,
            hasher,
        }
    }
}

impl Default for ProdVerifier<ProdVerifierComponents> {
    fn default() -> Self {
        Self::new(
            ProdPredicates::default(),
            ProdVotingPowerCalculator::default(),
            ProdCommitValidator::default(),
            ProdHasher::default(),
        )
    }
}

impl<C: VerifierComponents> Verifier for ProdVerifier<C> {
    fn verify(
        &self,
        untrusted: &LightBlock,
        trusted: &LightBlock,
        options: &Options,
        now: Time,
    ) -> Verdict {
        preds::verify(
            &self.predicates,
            &self.voting_power_calculator,
            &self.commit_validator,
            &self.hasher,
            trusted,
            untrusted,
            options,
            now,
        )
        .into()
    }
}
