use crate::prelude::*;

pub mod errors;
pub mod production;

pub trait VerificationPredicates {
    fn validator_sets_match(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<(), VerificationError>;

    fn next_validators_match(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<(), VerificationError>;

    fn header_matches_commit(
        &self,
        header: &Header,
        commit: &Commit,
        header_hasher: &dyn HeaderHasher,
    ) -> Result<(), VerificationError>;

    fn valid_commit(
        &self,
        commit: &Commit,
        validators: &ValidatorSet,
        validator: &dyn CommitValidator,
    ) -> Result<(), VerificationError>;

    fn is_within_trust_period(
        &self,
        header: &Header,
        trusting_period: Duration,
        now: Time,
    ) -> Result<(), VerificationError>;

    fn is_monotonic_bft_time(
        &self,
        untrusted_header: &Header,
        trusted_header: &Header,
    ) -> Result<(), VerificationError>;

    fn is_monotonic_height(
        &self,
        untrusted_header: &Header,
        trusted_header: &Header,
    ) -> Result<(), VerificationError>;

    fn has_sufficient_voting_power(
        &self,
        commit: &Commit,
        validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError>;

    fn has_sufficient_validators_overlap(
        &self,
        untrusted_commit: &Commit,
        trusted_validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError>;

    fn has_sufficient_signers_overlap(
        &self,
        untrusted_commit: &Commit,
        untrusted_validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError>;

    fn valid_next_validator_set(
        &self,
        untrusted_sh: &SignedHeader,
        untrusted_next_vals: &ValidatorSet,
    ) -> Result<(), VerificationError>;
}

pub fn validate_light_block(
    vp: &dyn VerificationPredicates,
    commit_validator: &dyn CommitValidator,
    header_hasher: &dyn HeaderHasher,
    trusted_state: &TrustedState,
    light_block: &LightBlock,
    options: VerificationOptions,
) -> Result<(), VerificationError> {
    let untrusted_sh = &light_block.signed_header;
    let untrusted_vals = &light_block.validators;
    let untrusted_next_vals = &light_block.next_validators;

    // Ensure the latest trusted header hasn't expired
    vp.is_within_trust_period(
        &trusted_state.header(),
        options.trusting_period,
        options.now,
    )?;

    // Ensure the header validator hashes match the given validators
    vp.validator_sets_match(&untrusted_sh, &untrusted_vals)?;

    // Ensure the header next validator hashes match the given next validators
    vp.next_validators_match(&untrusted_sh, &untrusted_next_vals)?;

    // Ensure the header matches the commit
    vp.header_matches_commit(&untrusted_sh.header, &untrusted_sh.commit, header_hasher)?;

    // Additional implementation specific validation
    vp.valid_commit(
        &untrusted_sh.commit,
        &untrusted_sh.validators,
        commit_validator,
    )?;

    vp.is_monotonic_bft_time(&untrusted_sh.header, &trusted_state.header())?;

    if untrusted_sh.header.height == trusted_state.header().height {
        vp.valid_next_validator_set(&untrusted_sh, &untrusted_next_vals)?;
    } else {
        vp.is_monotonic_height(&trusted_state.header(), &untrusted_sh.header)?;
    }

    Ok(())
}

pub fn verify_overlap(
    vp: &dyn VerificationPredicates,
    voting_power_calculator: &dyn VotingPowerCalculator,
    trusted_state: &TrustedState,
    light_block: &LightBlock,
    options: VerificationOptions,
) -> Result<(), VerificationError> {
    let untrusted_sh = &light_block.signed_header;
    let untrusted_vals = &light_block.validators;

    vp.has_sufficient_validators_overlap(
        &untrusted_sh.commit,
        &trusted_state.validators,
        &options.trust_threshold,
        voting_power_calculator,
    )?;

    vp.has_sufficient_signers_overlap(
        &untrusted_sh.commit,
        &untrusted_vals,
        &options.trust_threshold,
        voting_power_calculator,
    )?;

    Ok(())
}

pub fn has_sufficient_voting_power(
    vp: &dyn VerificationPredicates,
    voting_power_calculator: &dyn VotingPowerCalculator,
    light_block: &LightBlock,
    options: VerificationOptions,
) -> Result<(), VerificationError> {
    let untrusted_sh = &light_block.signed_header;

    vp.has_sufficient_voting_power(
        &untrusted_sh.commit,
        &untrusted_sh.validators,
        &options.trust_threshold,
        voting_power_calculator,
    )
}
