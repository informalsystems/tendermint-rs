use std::time::{Duration, SystemTime};

use crate::prelude::*;

pub mod production;

pub trait VerificationPredicates {
    fn validator_sets_match(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<(), Error>;

    fn next_validators_match(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<(), Error>;

    fn header_matches_commit(
        &self,
        header: &Header,
        commit: &Commit,
        header_hasher: impl HeaderHasher,
    ) -> Result<(), Error>;

    fn valid_commit(
        &self,
        commit: &Commit,
        validators: &ValidatorSet,
        validator: impl CommitValidator,
    ) -> Result<(), Error>;

    fn is_within_trust_period(
        &self,
        header: &Header,
        trusting_period: Duration,
        now: SystemTime,
    ) -> Result<(), Error>;

    fn is_monotonic_bft_time(
        &self,
        untrusted_header: &Header,
        trusted_header: &Header,
    ) -> Result<(), Error>;

    fn is_monotonic_height(
        &self,
        untrusted_header: &Header,
        trusted_header: &Header,
    ) -> Result<(), Error>;

    fn has_sufficient_voting_power(
        &self,
        commit: &Commit,
        validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &impl VotingPowerCalculator,
    ) -> Result<(), Error>;

    fn has_sufficient_validators_overlap(
        &self,
        untrusted_commit: &Commit,
        trusted_validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &impl VotingPowerCalculator,
    ) -> Result<(), Error>;

    fn has_sufficient_signers_overlap(
        &self,
        untrusted_commit: &Commit,
        untrusted_validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &impl VotingPowerCalculator,
    ) -> Result<(), Error>;

    fn valid_next_validator_set(
        &self,
        untrusted_sh: &SignedHeader,
        untrusted_next_vals: &ValidatorSet,
    ) -> Result<(), Error>;

    fn verify_untrusted_light_block(
        &self,
        voting_power_calculator: impl VotingPowerCalculator,
        commit_validator: impl CommitValidator,
        header_hasher: impl HeaderHasher,
        trusted_state: &TrustedState,
        light_block: &LightBlock,
        trust_threshold: &TrustThreshold,
        trusting_period: Duration,
        now: SystemTime,
    ) -> Result<(), Error> {
        let untrusted_sh = &light_block.signed_header;
        let untrusted_vals = &light_block.validator_set;
        let untrusted_next_vals = &light_block.next_validator_set;

        // Ensure the latest trusted header hasn't expired
        self.is_within_trust_period(&trusted_state.header, trusting_period, now)?;

        // Ensure the header validator hashes match the given validators
        self.validator_sets_match(&untrusted_sh, &untrusted_vals)?;

        // Ensure the header next validator hashes match the given next validators
        self.next_validators_match(&untrusted_sh, &untrusted_next_vals)?;

        // Ensure the header matches the commit
        self.header_matches_commit(&untrusted_sh.header, &untrusted_sh.commit, &header_hasher)?;

        // Additional implementation specific validation
        self.valid_commit(
            &untrusted_sh.commit,
            &untrusted_sh.validators,
            &commit_validator,
        )?;

        self.is_monotonic_bft_time(&untrusted_sh.header, &trusted_state.header)?;

        if untrusted_sh.header.height == trusted_state.header.height {
            self.valid_next_validator_set(&untrusted_sh, &untrusted_next_vals)?;
        } else if untrusted_sh.header.height > trusted_state.header.height {
            self.has_sufficient_voting_power(
                &untrusted_sh.commit,
                &untrusted_sh.validators,
                &trust_threshold,
                &voting_power_calculator,
            )?;
        } else {
            self.is_monotonic_height(&trusted_state.header, &untrusted_sh.header)?;

            // Ensure that the check above will always fail.
            unreachable!();
        }

        // All validation passed successfully.
        // Verify the validators correctly committed the block.

        self.has_sufficient_validators_overlap(
            &untrusted_sh.commit,
            &trusted_state.validators,
            &trust_threshold,
            &voting_power_calculator,
        )?;

        self.has_sufficient_signers_overlap(
            &untrusted_sh.commit,
            &untrusted_vals,
            &trust_threshold,
            &voting_power_calculator,
        )?;

        Ok(())
    }
}
