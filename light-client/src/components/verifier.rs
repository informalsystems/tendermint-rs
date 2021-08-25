//! Provides an interface and default implementation of the `Verifier` component

use crate::operations::voting_power::VotingPowerTally;
use crate::predicates as preds;
use crate::{
    errors::ErrorExt,
    light_client::Options,
    operations::{
        CommitValidator, ProdCommitValidator, ProdVotingPowerCalculator, VotingPowerCalculator,
    },
    types::{LightBlock, Time},
};
use preds::errors::{VerificationError, VerificationErrorDetail};
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

/// Production implementation of [`Verifier`].
pub type ProdVerifier = VerifierImpl<ProdVotingPowerCalculator, ProdCommitValidator>;

/// Production implementation of the verifier.
///
/// For testing purposes, this implementation is parametrized by:
/// - A set of predicates used to validate a light block
/// - A voting power calculator
/// - A commit validator
/// - A header hasher
///
/// For regular use, one can construct a standard implementation with `ProdVerifier::default()`.
pub struct VerifierImpl<C, V> {
    voting_power_calculator: C,
    commit_validator: V,
}

impl<C, V> VerifierImpl<C, V>
where
    C: VotingPowerCalculator,
    V: CommitValidator,
{
    /// Constructs a new instance of this struct
    pub fn new(voting_power_calculator: C, commit_validator: V) -> Self {
        Self {
            voting_power_calculator,
            commit_validator,
        }
    }
}

impl Default for ProdVerifier {
    fn default() -> Self {
        Self::new(
            ProdVotingPowerCalculator::default(),
            ProdCommitValidator::default(),
        )
    }
}

impl<C, V> Verifier for VerifierImpl<C, V>
where
    C: VotingPowerCalculator,
    V: CommitValidator,
{
    fn verify(
        &self,
        untrusted: &LightBlock,
        trusted: &LightBlock,
        options: &Options,
        now: Time,
    ) -> Verdict {
        preds::verify(
            &self.voting_power_calculator,
            &self.commit_validator,
            trusted,
            untrusted,
            options,
            now,
        )
        .into()
    }
}
