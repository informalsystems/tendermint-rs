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

    fn is_monotonic_bft_time(&self, header_a: &Header, header_b: &Header) -> Result<(), Error>;

    fn is_monotonic_height(&self, header_a: &Header, header_b: &Header) -> Result<(), Error>;

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
        trusted_state: &TrustedState,
        untrusted_sh: &SignedHeader,
        untrusted_next_vals: &ValidatorSet,
    ) -> Result<(), Error>;
}

pub fn verify_untrusted_light_block(
    pred: impl VerificationPredicates,
    voting_power_calculator: impl VotingPowerCalculator,
    commit_validator: impl CommitValidator,
    header_hasher: impl HeaderHasher,
    trusted_state: &TrustedState,
    untrusted_sh: &SignedHeader,
    untrusted_vals: &ValidatorSet,
    untrusted_next_vals: &ValidatorSet,
    trust_threshold: &TrustThreshold,
) -> Result<(), Error> {
    pred.validator_sets_match(&untrusted_sh, &untrusted_vals)?;
    pred.next_validators_match(&untrusted_sh, &untrusted_next_vals)?;

    pred.header_matches_commit(&untrusted_sh.header, &untrusted_sh.commit, &header_hasher)?;

    pred.valid_commit(
        &untrusted_sh.commit,
        &untrusted_sh.validators,
        &commit_validator,
    )?;

    pred.is_monotonic_bft_time(&untrusted_sh.header, &trusted_state.header)?;

    pred.is_monotonic_height(&trusted_state.header, &untrusted_sh.header)?;

    pred.valid_next_validator_set(&trusted_state, &untrusted_sh, &untrusted_next_vals)?;

    pred.has_sufficient_validators_overlap(
        &untrusted_sh.commit,
        &trusted_state.validators,
        &trust_threshold,
        &voting_power_calculator,
    )?;

    pred.has_sufficient_signers_overlap(
        &untrusted_sh.commit,
        &untrusted_vals,
        &trust_threshold,
        &voting_power_calculator,
    )?;

    Ok(())
}
