//! Predicates for light block validation and verification.

use crate::{
    ensure,
    light_client::Options,
    operations::{CommitValidator, Hasher, VotingPowerCalculator},
    types::{Commit, LightBlock, Time, TrustThreshold},
};

use errors::VerificationError;

use std::marker::PhantomData;
use std::time::Duration;

pub mod errors;

/// Production predicates, using the default implementation
/// of the `VerificationPredicates` trait.
#[derive(Copy, Clone, Debug)]
pub struct ProdPredicates<LB> {
    marker: PhantomData<LB>,
}

impl<LB> VerificationPredicates<LB> for ProdPredicates<LB> where LB: LightBlock {}

impl<LB> Default for ProdPredicates<LB> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

/// Defines the various predicates used to validate and verify light blocks.
///
/// A default, spec abiding implementation is provided for each method.
///
/// This enables test implementations to only override a single method rather than
/// have to re-define every predicate.
pub trait VerificationPredicates<LB: LightBlock>: Send {
    fn validator_sets_match(
        &self,
        light_block: &LB,
        hasher: &dyn Hasher<LB>,
    ) -> Result<(), VerificationError> {
        let validators_hash = hasher.hash_validator_set(&light_block.validators());

        ensure!(
            light_block.validators_hash() == validators_hash,
            VerificationError::InvalidValidatorSet {
                header_validators_hash: light_block.validators_hash(),
                validators_hash,
            }
        );

        Ok(())
    }

    fn next_validators_match(
        &self,
        light_block: &LB,
        hasher: &dyn Hasher<LB>,
    ) -> Result<(), VerificationError> {
        let next_validators_hash = hasher.hash_validator_set(&light_block.next_validators());

        ensure!(
            light_block.next_validators_hash() == next_validators_hash,
            VerificationError::InvalidNextValidatorSet {
                header_next_validators_hash: light_block.next_validators_hash(),
                next_validators_hash,
            }
        );

        Ok(())
    }

    fn header_matches_commit(
        &self,
        header: &LB::Header,
        commit: &LB::Commit,
        hasher: &dyn Hasher<LB>,
    ) -> Result<(), VerificationError> {
        let header_hash = hasher.hash_header(&header);

        ensure!(
            header_hash == commit.block_hash(),
            VerificationError::InvalidCommitValue {
                header_hash,
                commit_hash: commit.block_hash(),
            }
        );

        Ok(())
    }

    fn valid_commit(
        &self,
        commit: &LB::Commit,
        validators: &LB::ValidatorSet,
        commit_validator: &dyn CommitValidator<LB>,
    ) -> Result<(), VerificationError> {
        commit_validator.validate(commit, validators)?;
        commit_validator.validate_full(commit, validators)?;

        Ok(())
    }

    fn is_within_trust_period(
        &self,
        light_block: &LB,
        trusting_period: Duration,
        clock_drift: Duration,
        now: Time,
    ) -> Result<(), VerificationError> {
        ensure!(
            light_block.header_time() < now + clock_drift,
            VerificationError::HeaderFromTheFuture {
                header_time: light_block.header_time(),
                now
            }
        );

        let expires_at = light_block.header_time() + trusting_period;
        ensure!(
            expires_at > now,
            VerificationError::NotWithinTrustPeriod {
                at: expires_at,
                now,
            }
        );

        Ok(())
    }

    fn is_monotonic_bft_time(&self, untrusted: &LB, trusted: &LB) -> Result<(), VerificationError> {
        ensure!(
            untrusted.header_time() > trusted.header_time(),
            VerificationError::NonMonotonicBftTime {
                header_bft_time: untrusted.header_time(),
                trusted_header_bft_time: trusted.header_time(),
            }
        );

        Ok(())
    }

    fn is_monotonic_height(&self, untrusted: &LB, trusted: &LB) -> Result<(), VerificationError> {
        ensure!(
            untrusted.height() > trusted.height(),
            VerificationError::NonIncreasingHeight {
                got: untrusted.height(),
                expected: trusted.height() + 1,
            }
        );

        Ok(())
    }

    fn has_sufficient_validators_overlap(
        &self,
        untrusted_header: &LB::SignedHeader,
        trusted_validators: &LB::ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &dyn VotingPowerCalculator<LB>,
    ) -> Result<(), VerificationError> {
        calculator.check_enough_trust(untrusted_header, trusted_validators, *trust_threshold)?;
        Ok(())
    }

    fn has_sufficient_signers_overlap(
        &self,
        untrusted_header: &LB::SignedHeader,
        untrusted_validators: &LB::ValidatorSet,
        calculator: &dyn VotingPowerCalculator<LB>,
    ) -> Result<(), VerificationError> {
        calculator.check_signers_overlap(untrusted_header, untrusted_validators)?;
        Ok(())
    }

    fn valid_next_validator_set(
        &self,
        light_block: &LB,
        trusted_state: &LB,
    ) -> Result<(), VerificationError> {
        ensure!(
            light_block.validators_hash() == trusted_state.next_validators_hash(),
            VerificationError::InvalidNextValidatorSet {
                header_next_validators_hash: light_block.validators_hash(),
                next_validators_hash: trusted_state.next_validators_hash(),
            }
        );

        Ok(())
    }
}

/// Validate the given light block.
///
/// - Ensure the latest trusted header hasn't expired
/// - Ensure the header validator hashes match the given validators
/// - Ensure the header next validator hashes match the given next validators
/// - Additional implementation specific validation via `commit_validator`
/// - Check that the untrusted block is more recent than the trusted state
/// - If the untrusted block is the very next block after the trusted block,
/// check that their (next) validator sets hashes match.
/// - Otherwise, ensure that the untrusted block has a greater height than
/// the trusted block.
#[allow(clippy::too_many_arguments)]
pub fn verify<LB: LightBlock>(
    vp: &dyn VerificationPredicates<LB>,
    voting_power_calculator: &dyn VotingPowerCalculator<LB>,
    commit_validator: &dyn CommitValidator<LB>,
    hasher: &dyn Hasher<LB>,
    trusted: &LB,
    untrusted: &LB,
    options: &Options,
    now: Time,
) -> Result<(), VerificationError> {
    // Ensure the latest trusted header hasn't expired
    vp.is_within_trust_period(trusted, options.trusting_period, options.clock_drift, now)?;

    // Ensure the header validator hashes match the given validators
    vp.validator_sets_match(&untrusted, &*hasher)?;

    // Ensure the header next validator hashes match the given next validators
    vp.next_validators_match(&untrusted, &*hasher)?;

    // Ensure the header matches the commit
    vp.header_matches_commit(&untrusted.header(), &untrusted.commit(), hasher)?;

    // Additional implementation specific validation
    vp.valid_commit(
        &untrusted.commit(),
        &untrusted.validators(),
        commit_validator,
    )?;

    // Check that the untrusted block is more recent than the trusted state
    vp.is_monotonic_bft_time(&untrusted, &trusted)?;

    let trusted_next_height = trusted.height().checked_add(1).expect("height overflow");

    if untrusted.height() == trusted_next_height {
        // If the untrusted block is the very next block after the trusted block,
        // check that their (next) validator sets hashes match.
        vp.valid_next_validator_set(&untrusted, trusted)?;
    } else {
        // Otherwise, ensure that the untrusted block has a greater height than
        // the trusted block.
        vp.is_monotonic_height(&untrusted, &trusted)?;

        // Check there is enough overlap between the validator sets of
        // the trusted and untrusted blocks.
        vp.has_sufficient_validators_overlap(
            &untrusted.signed_header(),
            &trusted.next_validators(),
            &options.trust_threshold,
            voting_power_calculator,
        )?;
    }

    // Verify that more than 2/3 of the validators correctly committed the block.
    vp.has_sufficient_signers_overlap(
        &untrusted.signed_header(),
        &untrusted.validators(),
        voting_power_calculator,
    )?;

    Ok(())
}
