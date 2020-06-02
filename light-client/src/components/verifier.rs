use crate::predicates as preds;
use crate::prelude::*;

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

impl Verdict {
    /// Execute the `next` action only if `self` is a successful verdict.
    /// This allows chaining computations returning a `Verdict` until
    /// on of them is not successful.
    pub fn and_then<A: Into<Self>>(self, next: impl Fn() -> A) -> Self {
        match self {
            Verdict::Success => next().into(),
            other => other,
        }
    }
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
pub trait Verifier {
    /// Perform the verification.
    fn verify(&self, untrusted: &LightBlock, trusted: &LightBlock, options: &Options) -> Verdict;
}

/// Production implementation of the verifier.
///
/// For testing purposes, this implementation is parametrized by:
/// - A set of predicates used to validate a light block
/// - A voting power calculator
/// - A commit validator
/// - A header hasher
///
/// For regular use, one can summon an implementation with `ProdVerifier::default()`.
pub struct ProdVerifier {
    predicates: Box<dyn VerificationPredicates>,
    voting_power_calculator: Box<dyn VotingPowerCalculator>,
    commit_validator: Box<dyn CommitValidator>,
    header_hasher: Box<dyn HeaderHasher>,
}

impl ProdVerifier {
    pub fn new(
        predicates: impl VerificationPredicates + 'static,
        voting_power_calculator: impl VotingPowerCalculator + 'static,
        commit_validator: impl CommitValidator + 'static,
        header_hasher: impl HeaderHasher + 'static,
    ) -> Self {
        Self {
            predicates: Box::new(predicates),
            voting_power_calculator: Box::new(voting_power_calculator),
            commit_validator: Box::new(commit_validator),
            header_hasher: Box::new(header_hasher),
        }
    }
}

impl Default for ProdVerifier {
    fn default() -> Self {
        Self::new(
            ProdPredicates,
            ProdVotingPowerCalculator,
            ProdCommitValidator,
            ProdHeaderHasher,
        )
    }
}

impl Verifier for ProdVerifier {
    fn verify(&self, untrusted: &LightBlock, trusted: &TrustedState, options: &Options) -> Verdict {
        let verdict: Verdict = preds::validate_light_block(
            &*self.predicates,
            &self.commit_validator,
            &self.header_hasher,
            &trusted,
            &untrusted,
            options,
        )
        .into();

        let verdict = verdict.and_then(|| {
            preds::verify_overlap(
                &*self.predicates,
                &self.voting_power_calculator,
                &trusted,
                &untrusted,
                options,
            )
        });

        let verdict = verdict.and_then(|| {
            preds::has_sufficient_voting_power(
                &*self.predicates,
                &self.voting_power_calculator,
                &untrusted,
                options,
            )
        });

        verdict
    }
}
