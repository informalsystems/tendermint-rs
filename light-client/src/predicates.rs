use crate::prelude::*;

use tendermint::lite::ValidatorSet as _;

pub mod errors;

#[derive(Copy, Clone, Debug)]
pub struct ProdPredicates;

impl VerificationPredicates for ProdPredicates {}

pub trait VerificationPredicates {
    fn validator_sets_match(&self, light_block: &LightBlock) -> Result<(), VerificationError> {
        ensure!(
            light_block.signed_header.header.validators_hash == light_block.validators.hash(),
            VerificationError::InvalidValidatorSet {
                header_validators_hash: light_block.signed_header.header.validators_hash,
                validators_hash: light_block.validators.hash(),
            }
        );

        Ok(())
    }

    fn next_validators_match(&self, light_block: &LightBlock) -> Result<(), VerificationError> {
        ensure!(
            light_block.signed_header.header.next_validators_hash
                == light_block.next_validators.hash(),
            VerificationError::InvalidNextValidatorSet {
                header_next_validators_hash: light_block.signed_header.header.next_validators_hash,
                next_validators_hash: light_block.next_validators.hash(),
            }
        );

        Ok(())
    }

    fn header_matches_commit(
        &self,
        signed_header: &SignedHeader,
        header_hasher: &dyn HeaderHasher,
    ) -> Result<(), VerificationError> {
        let header_hash = header_hasher.hash(&signed_header.header);

        ensure!(
            header_hash == signed_header.commit.block_id.hash,
            VerificationError::InvalidCommitValue {
                header_hash,
                commit_hash: signed_header.commit.block_id.hash,
            }
        );

        Ok(())
    }

    fn valid_commit(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
        validator: &dyn CommitValidator,
    ) -> Result<(), VerificationError> {
        // FIXME: Do not discard underlying error
        validator
            .validate(signed_header, validators)
            .map_err(|e| VerificationError::InvalidCommit(e.to_string()))?;

        Ok(())
    }

    fn is_within_trust_period(
        &self,
        header: &Header,
        trusting_period: Duration,
        clock_drift: Duration,
        now: Time,
    ) -> Result<(), VerificationError> {
        ensure!(
            header.time < now + clock_drift,
            VerificationError::HeaderFromTheFuture {
                header_time: header.time,
                now
            }
        );

        let expires_at = header.time + trusting_period;
        ensure!(
            expires_at > now,
            VerificationError::NotWithinTrustPeriod {
                at: expires_at,
                now,
            }
        );

        Ok(())
    }

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

    fn is_monotonic_height(
        &self,
        untrusted_header: &Header,
        trusted_header: &Header,
    ) -> Result<(), VerificationError> {
        let trusted_height: Height = trusted_header.height.into();

        ensure!(
            untrusted_header.height > trusted_header.height,
            VerificationError::NonIncreasingHeight {
                got: untrusted_header.height.into(),
                expected: trusted_height + 1,
            }
        );

        Ok(())
    }

    fn has_sufficient_voting_power(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError> {
        // FIXME: Do not discard underlying error
        let total_power = calculator.total_power_of(validators);
        let voting_power = calculator
            .voting_power_in(signed_header, validators)
            .map_err(|e| VerificationError::ImplementationSpecific(e.to_string()))?;

        ensure!(
            voting_power * trust_threshold.denominator > total_power * trust_threshold.numerator,
            VerificationError::InsufficientVotingPower {
                total_power,
                voting_power,
            }
        );

        Ok(())
    }

    fn has_sufficient_validators_overlap(
        &self,
        untrusted_sh: &SignedHeader,
        trusted_validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError> {
        // FIXME: Do not discard underlying error
        let total_power = calculator.total_power_of(trusted_validators);
        let voting_power = calculator
            .voting_power_in(untrusted_sh, trusted_validators)
            .map_err(|e| VerificationError::ImplementationSpecific(e.to_string()))?;

        ensure!(
            voting_power * trust_threshold.denominator > total_power * trust_threshold.numerator,
            VerificationError::InsufficientValidatorsOverlap {
                total_power,
                signed_power: voting_power,
            }
        );

        Ok(())
    }

    fn has_sufficient_signers_overlap(
        &self,
        untrusted_sh: &SignedHeader,
        untrusted_validators: &ValidatorSet,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError> {
        let total_power = calculator.total_power_of(untrusted_validators);
        let signed_power = calculator
            .voting_power_in(untrusted_sh, untrusted_validators)
            .map_err(|e| VerificationError::ImplementationSpecific(e.to_string()))?;

        ensure!(
            signed_power * 3 > total_power * 2,
            VerificationError::InsufficientCommitPower {
                total_power,
                signed_power,
            }
        );

        Ok(())
    }

    fn valid_next_validator_set(
        &self,
        light_block: &LightBlock,
        trusted_state: &TrustedState,
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

pub fn validate_light_block(
    vp: &dyn VerificationPredicates,
    commit_validator: &dyn CommitValidator,
    header_hasher: &dyn HeaderHasher,
    trusted_state: &TrustedState,
    light_block: &LightBlock,
    options: &Options,
) -> Result<(), VerificationError> {
    // Ensure the latest trusted header hasn't expired
    vp.is_within_trust_period(
        &trusted_state.signed_header.header,
        options.trusting_period,
        options.clock_drift,
        options.now,
    )?;

    // Ensure the header validator hashes match the given validators
    vp.validator_sets_match(&light_block)?;

    // Ensure the header next validator hashes match the given next validators
    vp.next_validators_match(&light_block)?;

    // Ensure the header matches the commit
    vp.header_matches_commit(&light_block.signed_header, header_hasher)?;

    // Additional implementation specific validation
    vp.valid_commit(
        &light_block.signed_header,
        &light_block.validators,
        commit_validator,
    )?;

    vp.is_monotonic_bft_time(
        &light_block.signed_header.header,
        &trusted_state.signed_header.header,
    )?;

    let trusted_state_next_height = trusted_state
        .height()
        .checked_add(1)
        .expect("height overflow");

    if light_block.height() == trusted_state_next_height {
        vp.valid_next_validator_set(&light_block, trusted_state)?;
    } else {
        vp.is_monotonic_height(
            &light_block.signed_header.header,
            &trusted_state.signed_header.header,
        )?;
    }

    Ok(())
}

pub fn verify_overlap(
    vp: &dyn VerificationPredicates,
    voting_power_calculator: &dyn VotingPowerCalculator,
    trusted_state: &TrustedState,
    light_block: &LightBlock,
    options: &Options,
) -> Result<(), VerificationError> {
    let untrusted_sh = &light_block.signed_header;
    let untrusted_vals = &light_block.validators;

    vp.has_sufficient_validators_overlap(
        &untrusted_sh,
        &trusted_state.next_validators,
        &options.trust_threshold,
        voting_power_calculator,
    )?;

    vp.has_sufficient_signers_overlap(&untrusted_sh, &untrusted_vals, voting_power_calculator)?;

    Ok(())
}

pub fn has_sufficient_voting_power(
    vp: &dyn VerificationPredicates,
    voting_power_calculator: &dyn VotingPowerCalculator,
    light_block: &LightBlock,
    options: &Options,
) -> Result<(), VerificationError> {
    let untrusted_sh = &light_block.signed_header;
    let untrusted_vals = &light_block.validators;

    vp.has_sufficient_voting_power(
        &untrusted_sh,
        &untrusted_vals,
        &options.trust_threshold,
        voting_power_calculator,
    )
}
