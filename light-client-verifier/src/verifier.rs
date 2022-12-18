//! Provides an interface and default implementation of the `Verifier` component

use preds::{ProdPredicates, VerificationPredicates};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{ErrorExt, VerificationError, VerificationErrorDetail},
    operations::{
        voting_power::VotingPowerTally, CommitValidator, Hasher, ProdCommitValidator, ProdHasher,
        ProdVotingPowerCalculator, VotingPowerCalculator,
    },
    options::Options,
    predicates as preds,
    types::{Time, TrustedBlockState, UntrustedBlockState},
};

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
        untrusted: UntrustedBlockState<'_>,
        trusted: TrustedBlockState<'_>,
        options: &Options,
        now: Time,
    ) -> Verdict;
}

macro_rules! verdict {
    ($e:expr) => {
        let result = $e;
        if result.is_err() {
            return result.into();
        }
    };
}

macro_rules! ensure_verdict_success {
    ($e:expr) => {
        let verdict = $e;
        if !matches!(verdict, Verdict::Success) {
            return verdict;
        }
    };
}

/// Predicate verifier encapsulating components necessary to facilitate
/// verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredicateVerifier<P, C, V, H> {
    predicates: P,
    voting_power_calculator: C,
    commit_validator: V,
    hasher: H,
}

impl<P, C, V, H> Default for PredicateVerifier<P, C, V, H>
where
    P: Default,
    C: Default,
    V: Default,
    H: Default,
{
    fn default() -> Self {
        Self {
            predicates: P::default(),
            voting_power_calculator: C::default(),
            commit_validator: V::default(),
            hasher: H::default(),
        }
    }
}

impl<P, C, V, H> PredicateVerifier<P, C, V, H>
where
    P: VerificationPredicates,
    C: VotingPowerCalculator,
    V: CommitValidator,
    H: Hasher,
{
    /// Constructor.
    pub fn new(predicates: P, voting_power_calculator: C, commit_validator: V, hasher: H) -> Self {
        Self {
            predicates,
            voting_power_calculator,
            commit_validator,
            hasher,
        }
    }

    /// Validates an `UntrustedBlockState`.
    pub fn verify_validator_sets(&self, untrusted: &UntrustedBlockState<'_>) -> Verdict {
        // Ensure the header validator hashes match the given validators
        verdict!(self.predicates.validator_sets_match(
            untrusted.validators,
            untrusted.signed_header.header.validators_hash,
            &self.hasher,
        ));

        // Ensure the header next validator hashes match the given next validators
        if let Some(untrusted_next_validators) = untrusted.next_validators {
            verdict!(self.predicates.next_validators_match(
                untrusted_next_validators,
                untrusted.signed_header.header.next_validators_hash,
                &self.hasher,
            ));
        }

        // Ensure the header matches the commit
        verdict!(self.predicates.header_matches_commit(
            &untrusted.signed_header.header,
            untrusted.signed_header.commit.block_id.hash,
            &self.hasher,
        ));

        // Additional implementation specific validation
        verdict!(self.predicates.valid_commit(
            untrusted.signed_header,
            untrusted.validators,
            &self.commit_validator,
        ));

        Verdict::Success
    }

    /// Verify that more than 2/3 of the validators correctly committed the block.
    pub fn verify_commit(&self, untrusted: &UntrustedBlockState<'_>) -> Verdict {
        verdict!(self.predicates.has_sufficient_signers_overlap(
            untrusted.signed_header,
            untrusted.validators,
            &self.voting_power_calculator,
        ));

        Verdict::Success
    }

    /// Validate an `UntrustedBlockState`, based on the given `TrustedBlockState`, `Options` and
    /// current time.
    pub fn validate_against_trusted(
        &self,
        untrusted: &UntrustedBlockState<'_>,
        trusted: &TrustedBlockState<'_>,
        options: &Options,
        now: Time,
    ) -> Verdict {
        // Ensure the latest trusted header hasn't expired
        verdict!(self.predicates.is_within_trust_period(
            trusted.header_time,
            options.trusting_period,
            now,
        ));

        // Ensure the header isn't from a future time
        verdict!(self.predicates.is_header_from_past(
            untrusted.signed_header.header.time,
            options.clock_drift,
            now,
        ));

        // Check that the untrusted block is more recent than the trusted state
        verdict!(self
            .predicates
            .is_monotonic_bft_time(untrusted.signed_header.header.time, trusted.header_time));

        // Check that the chain-id of the untrusted block matches that of the trusted state
        verdict!(self
            .predicates
            .is_matching_chain_id(&untrusted.signed_header.header.chain_id, trusted.chain_id));

        let trusted_next_height = trusted.height.increment();

        if untrusted.height() == trusted_next_height {
            // If the untrusted block is the very next block after the trusted block,
            // check that their (next) validator sets hashes match.
            verdict!(self.predicates.valid_next_validator_set(
                untrusted.signed_header.header.validators_hash,
                trusted.next_validators_hash,
            ));
        } else {
            // Otherwise, ensure that the untrusted block has a greater height than
            // the trusted block.
            verdict!(self
                .predicates
                .is_monotonic_height(untrusted.signed_header.header.height, trusted.height));
        }

        Verdict::Success
    }

    /// Check there is enough overlap between the validator sets of the trusted and untrusted
    /// blocks.
    pub fn verify_commit_against_trusted(
        &self,
        untrusted: &UntrustedBlockState<'_>,
        trusted: &TrustedBlockState<'_>,
        options: &Options,
    ) -> Verdict {
        let trusted_next_height = trusted.height.increment();

        if untrusted.height() != trusted_next_height {
            // Check there is enough overlap between the validator sets of
            // the trusted and untrusted blocks.
            verdict!(self.predicates.has_sufficient_validators_overlap(
                untrusted.signed_header,
                trusted.next_validators,
                &options.trust_threshold,
                &self.voting_power_calculator,
            ));
        }

        Verdict::Success
    }
}

impl<P, C, V, H> Verifier for PredicateVerifier<P, C, V, H>
where
    P: VerificationPredicates,
    C: VotingPowerCalculator,
    V: CommitValidator,
    H: Hasher,
{
    /// Validate the given light block state by performing the following checks ->
    ///
    /// - Validate the untrusted header
    ///     - Ensure the header validator hashes match the given validators
    ///     - Ensure the header next validator hashes match the given next validators
    ///     - Ensure the header matches the commit
    ///     - Ensure commit is valid
    /// - Validate the untrusted header against the trusted header
    ///     - Ensure the latest trusted header hasn't expired
    ///     - Ensure the header isn't from a future time
    ///     - Check that the untrusted block is more recent than the trusted state
    ///     - If the untrusted block is the very next block after the trusted block, check that
    ///       their (next) validator sets hashes match.
    ///     - Otherwise, ensure that the untrusted block has a greater height than the trusted
    ///       block.
    /// - Check there is enough overlap between the validator sets of the trusted and untrusted
    ///   blocks.
    /// - Verify that more than 2/3 of the validators correctly committed the block.
    ///
    /// **NOTE**: If the untrusted state's `next_validators` field is `None`,
    /// this will not (and will not be able to) check whether the untrusted
    /// state's `next_validators_hash` field is valid.
    fn verify(
        &self,
        untrusted: UntrustedBlockState<'_>,
        trusted: TrustedBlockState<'_>,
        options: &Options,
        now: Time,
    ) -> Verdict {
        ensure_verdict_success!(self.verify_validator_sets(&untrusted));
        ensure_verdict_success!(self.validate_against_trusted(&untrusted, &trusted, options, now));
        ensure_verdict_success!(self.verify_commit_against_trusted(&untrusted, &trusted, options));
        ensure_verdict_success!(self.verify_commit(&untrusted));
        Verdict::Success
    }
}

/// The default production implementation of the [`PredicateVerifier`].
pub type ProdVerifier =
    PredicateVerifier<ProdPredicates, ProdVotingPowerCalculator, ProdCommitValidator, ProdHasher>;

#[cfg(test)]
mod tests {
    use alloc::{borrow::ToOwned, string::ToString};
    use core::{ops::Sub, time::Duration};

    use tendermint::Time;
    use tendermint_testgen::{light_block::LightBlock as TestgenLightBlock, Generator};

    use crate::{
        errors::VerificationErrorDetail, options::Options, types::LightBlock, ProdVerifier,
        Verdict, Verifier,
    };

    #[test]
    fn test_verification_failure_on_chain_id_mismatch() {
        let now = Time::now();

        // Create a default light block with a valid chain-id for height `1` with a timestamp 20
        // secs before now (to be treated as trusted state)
        let light_block_1: LightBlock = TestgenLightBlock::new_default_with_time_and_chain_id(
            "chain-1".to_owned(),
            now.sub(Duration::from_secs(20)).unwrap(),
            1u64,
        )
        .generate()
        .unwrap()
        .into();

        // Create another default block with a different chain-id for height `2` with a timestamp 10
        // secs before now (to be treated as untrusted state)
        let light_block_2: LightBlock = TestgenLightBlock::new_default_with_time_and_chain_id(
            "forged-chain".to_owned(),
            now.sub(Duration::from_secs(10)).unwrap(),
            2u64,
        )
        .generate()
        .unwrap()
        .into();

        let vp = ProdVerifier::default();
        let opt = Options {
            trust_threshold: Default::default(),
            trusting_period: Duration::from_secs(60),
            clock_drift: Default::default(),
        };

        let verdict = vp.verify(
            light_block_2.as_untrusted_state(),
            light_block_1.as_trusted_state(),
            &opt,
            Time::now(),
        );

        match verdict {
            Verdict::Invalid(VerificationErrorDetail::ChainIdMismatch(e)) => {
                let chain_id_1 = light_block_1.signed_header.header.chain_id;
                let chain_id_2 = light_block_2.signed_header.header.chain_id;
                assert_eq!(e.got, chain_id_2.to_string());
                assert_eq!(e.expected, chain_id_1.to_string());
            },
            v => panic!("expected ChainIdMismatch error, got: {:?}", v),
        }
    }
}
