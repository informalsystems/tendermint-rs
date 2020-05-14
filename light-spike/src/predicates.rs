use crate::prelude::*;

pub mod errors;
pub mod production;

pub trait VerificationPredicates {
    fn validator_sets_match(&self, light_block: &LightBlock) -> Result<(), VerificationError>;

    fn next_validators_match(&self, light_block: &LightBlock) -> Result<(), VerificationError>;

    fn header_matches_commit(
        &self,
        signed_header: &SignedHeader,
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
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError>;

    fn has_sufficient_validators_overlap(
        &self,
        untrusted_sh: &SignedHeader,
        trusted_validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError>;

    fn has_sufficient_signers_overlap(
        &self,
        untrusted_sh: &SignedHeader,
        untrusted_validators: &ValidatorSet,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError>;

    fn valid_next_validator_set(
        &self,
        light_block: &LightBlock,
        trusted_state: &TrustedState,
    ) -> Result<(), VerificationError>;
}

pub fn validate_light_block(
    vp: &dyn VerificationPredicates,
    commit_validator: &dyn CommitValidator,
    header_hasher: &dyn HeaderHasher,
    trusted_state: &TrustedState,
    light_block: &LightBlock,
    options: &VerificationOptions,
) -> Result<(), VerificationError> {
    // Ensure the latest trusted header hasn't expired
    vp.is_within_trust_period(
        &trusted_state.signed_header.header,
        options.trusting_period,
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
        &light_block.signed_header.commit,
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
    options: &VerificationOptions,
) -> Result<(), VerificationError> {
    let untrusted_sh = &light_block.signed_header;
    let untrusted_vals = &light_block.validators;

    vp.has_sufficient_validators_overlap(
        &untrusted_sh,
        &trusted_state.validators,
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
    options: &VerificationOptions,
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
