use crate::{
    errors::ErrorExt,
    light_client::Options,
    operations::{
        CommitValidator, Hasher, ProdCommitValidator, ProdHasher, ProdVotingPowerCalculator,
        VotingPowerCalculator,
    },
    predicates::{errors::VerificationError, ProdPredicates, VerificationPredicates},
    types::{LightBlock, TMLightBlock, Time},
};

/// Represents the result of the verification performed by the
/// verifier component.
#[derive(Debug)]
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
pub trait Verifier<LB: LightBlock>: Send {
    /// Perform the verification.
    fn verify(&self, untrusted: &LB, trusted: &LB, options: &Options, now: Time) -> Verdict;
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
pub struct ProdVerifier<LB>
where
    LB: LightBlock,
{
    predicates: Box<dyn VerificationPredicates<LB>>,
    voting_power_calculator: Box<dyn VotingPowerCalculator<LB>>,
    commit_validator: Box<dyn CommitValidator<LB>>,
    hasher: Box<dyn Hasher<LB>>,
}

impl<LB> ProdVerifier<LB>
where
    LB: LightBlock,
{
    pub fn new(
        predicates: impl VerificationPredicates<LB> + 'static,
        voting_power_calculator: impl VotingPowerCalculator<LB> + 'static,
        commit_validator: impl CommitValidator<LB> + 'static,
        hasher: impl Hasher<LB> + 'static,
    ) -> Self {
        Self {
            predicates: Box::new(predicates),
            voting_power_calculator: Box::new(voting_power_calculator),
            commit_validator: Box::new(commit_validator),
            hasher: Box::new(hasher),
        }
    }
}

impl Default for ProdVerifier<TMLightBlock> {
    fn default() -> Self {
        Self::new(
            ProdPredicates::default(),
            ProdVotingPowerCalculator::default(),
            ProdCommitValidator::default(),
            ProdHasher::default(),
        )
    }
}

impl<LB> Verifier<LB> for ProdVerifier<LB>
where
    LB: LightBlock,
{
    fn verify(&self, untrusted: &LB, trusted: &LB, options: &Options, now: Time) -> Verdict {
        crate::predicates::verify(
            &*self.predicates,
            &*self.voting_power_calculator,
            &*self.commit_validator,
            &*self.hasher,
            &trusted,
            &untrusted,
            options,
            now,
        )
        .into()
    }
}
