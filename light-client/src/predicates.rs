//! Predicates for light block validation and verification.

use crate::{
    ensure,
    light_client::Options,
    operations::{CommitValidator, Hasher, VotingPowerCalculator},
    types::{Header, LightBlock, SignedHeader, Time, TrustThreshold, ValidatorSet},
};

use errors::VerificationError;
use std::time::Duration;

pub mod errors;

/// Production predicates, using the default implementation
/// of the `VerificationPredicates` trait.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProdPredicates;
impl VerificationPredicates for ProdPredicates {}

/// Defines the various predicates used to validate and verify light blocks.
///
/// A default, spec abiding implementation is provided for each method.
///
/// This enables test implementations to only override a single method rather than
/// have to re-define every predicate.
pub trait VerificationPredicates: Send {
    /// Compare the provided validator_set_hash against the hash produced from hashing the validator
    /// set.
    fn validator_sets_match(
        &self,
        light_block: &LightBlock,
        hasher: &dyn Hasher,
    ) -> Result<(), VerificationError> {
        let validators_hash = hasher.hash_validator_set(&light_block.validators);

        ensure!(
            light_block.signed_header.header.validators_hash == validators_hash,
            VerificationError::InvalidValidatorSet {
                header_validators_hash: light_block.signed_header.header.validators_hash,
                validators_hash,
            }
        );

        Ok(())
    }

    /// Check that the hash of the next validator set in the header match the actual one.
    fn next_validators_match(
        &self,
        light_block: &LightBlock,
        hasher: &dyn Hasher,
    ) -> Result<(), VerificationError> {
        let next_validators_hash = hasher.hash_validator_set(&light_block.next_validators);

        ensure!(
            light_block.signed_header.header.next_validators_hash == next_validators_hash,
            VerificationError::InvalidNextValidatorSet {
                header_next_validators_hash: light_block.signed_header.header.next_validators_hash,
                next_validators_hash,
            }
        );

        Ok(())
    }

    /// Check that the hash of the header in the commit matches the actual one.
    fn header_matches_commit(
        &self,
        signed_header: &SignedHeader,
        hasher: &dyn Hasher,
    ) -> Result<(), VerificationError> {
        let header_hash = hasher.hash_header(&signed_header.header);

        ensure!(
            header_hash == signed_header.commit.block_id.hash,
            VerificationError::InvalidCommitValue {
                header_hash,
                commit_hash: signed_header.commit.block_id.hash,
            }
        );

        Ok(())
    }

    /// Validate the commit using the given commit validator.
    fn valid_commit(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
        commit_validator: &dyn CommitValidator,
    ) -> Result<(), VerificationError> {
        commit_validator.validate(signed_header, validators)?;
        commit_validator.validate_full(signed_header, validators)?;

        Ok(())
    }

    /// Check that the trusted header is within the trusting period, adjusting for clock drift.
    fn is_within_trust_period(
        &self,
        trusted_header: &Header,
        trusting_period: Duration,
        now: Time,
    ) -> Result<(), VerificationError> {
        let expires_at = trusted_header.time + trusting_period;
        ensure!(
            expires_at > now,
            VerificationError::NotWithinTrustPeriod { expires_at, now }
        );

        Ok(())
    }

    /// Check that the untrusted header is from past.
    fn is_header_from_past(
        &self,
        untrusted_header: &Header,
        clock_drift: Duration,
        now: Time,
    ) -> Result<(), VerificationError> {
        ensure!(
            untrusted_header.time < now + clock_drift,
            VerificationError::HeaderFromTheFuture {
                header_time: untrusted_header.time,
                now
            }
        );

        Ok(())
    }

    /// Check that time passed monotonically between the trusted header and the untrusted one.
    fn is_monotonic_bft_time(
        &self,
        untrusted_header: &Header,
        trusted_header: &Header,
    ) -> Result<(), VerificationError> {
        ensure!(
            untrusted_header.time > trusted_header.time,
            VerificationError::NonMonotonicBftTime {
                header_bft_time: untrusted_header.time,
                trusted_header_bft_time: trusted_header.time,
            }
        );

        Ok(())
    }

    /// Check that the height increased between the trusted header and the untrusted one.
    fn is_monotonic_height(
        &self,
        untrusted_header: &Header,
        trusted_header: &Header,
    ) -> Result<(), VerificationError> {
        let trusted_height = trusted_header.height;

        ensure!(
            untrusted_header.height > trusted_header.height,
            VerificationError::NonIncreasingHeight {
                got: untrusted_header.height,
                expected: trusted_height.increment(),
            }
        );

        Ok(())
    }

    /// Check that there is enough validators overlap between the trusted validator set
    /// and the untrusted signed header.
    fn has_sufficient_validators_overlap(
        &self,
        untrusted_sh: &SignedHeader,
        trusted_validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError> {
        calculator.check_enough_trust(untrusted_sh, trusted_validators, *trust_threshold)?;
        Ok(())
    }

    /// Check that there is enough signers overlap between the given, untrusted validator set
    /// and the untrusted signed header.
    fn has_sufficient_signers_overlap(
        &self,
        untrusted_sh: &SignedHeader,
        untrusted_validators: &ValidatorSet,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError> {
        calculator.check_signers_overlap(untrusted_sh, untrusted_validators)?;
        Ok(())
    }

    /// Check that the hash of the next validator set in the trusted block matches
    /// the hash of the validator set in the untrusted one.
    fn valid_next_validator_set(
        &self,
        light_block: &LightBlock,
        trusted_state: &LightBlock,
    ) -> Result<(), VerificationError> {
        ensure!(
            light_block.signed_header.header.validators_hash
                == trusted_state.signed_header.header.next_validators_hash,
            VerificationError::InvalidNextValidatorSet {
                header_next_validators_hash: light_block.signed_header.header.validators_hash,
                next_validators_hash: trusted_state.signed_header.header.next_validators_hash,
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
pub fn verify(
    vp: &dyn VerificationPredicates,
    voting_power_calculator: &dyn VotingPowerCalculator,
    commit_validator: &dyn CommitValidator,
    hasher: &dyn Hasher,
    trusted: &LightBlock,
    untrusted: &LightBlock,
    options: &Options,
    now: Time,
) -> Result<(), VerificationError> {
    // Ensure the latest trusted header hasn't expired
    vp.is_within_trust_period(&trusted.signed_header.header, options.trusting_period, now)?;

    // Ensure the header isn't from a future time
    vp.is_header_from_past(&untrusted.signed_header.header, options.clock_drift, now)?;

    // Ensure the header validator hashes match the given validators
    vp.validator_sets_match(&untrusted, &*hasher)?;

    // Ensure the header next validator hashes match the given next validators
    vp.next_validators_match(&untrusted, &*hasher)?;

    // Ensure the header matches the commit
    vp.header_matches_commit(&untrusted.signed_header, hasher)?;

    // Additional implementation specific validation
    vp.valid_commit(
        &untrusted.signed_header,
        &untrusted.validators,
        commit_validator,
    )?;

    // Check that the untrusted block is more recent than the trusted state
    vp.is_monotonic_bft_time(
        &untrusted.signed_header.header,
        &trusted.signed_header.header,
    )?;

    let trusted_next_height = trusted.height().increment();

    if untrusted.height() == trusted_next_height {
        // If the untrusted block is the very next block after the trusted block,
        // check that their (next) validator sets hashes match.
        vp.valid_next_validator_set(&untrusted, trusted)?;
    } else {
        // Otherwise, ensure that the untrusted block has a greater height than
        // the trusted block.
        vp.is_monotonic_height(
            &untrusted.signed_header.header,
            &trusted.signed_header.header,
        )?;

        // Check there is enough overlap between the validator sets of
        // the trusted and untrusted blocks.
        vp.has_sufficient_validators_overlap(
            &untrusted.signed_header,
            &trusted.next_validators,
            &options.trust_threshold,
            voting_power_calculator,
        )?;
    }

    // Verify that more than 2/3 of the validators correctly committed the block.
    vp.has_sufficient_signers_overlap(
        &untrusted.signed_header,
        &untrusted.validators,
        voting_power_calculator,
    )?;

    Ok(())
}
