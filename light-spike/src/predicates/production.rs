use super::VerificationPredicates;
use crate::prelude::*;

pub struct ProductionPredicates;

impl VerificationPredicates for ProductionPredicates {
    fn validator_sets_match(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<(), Error> {
        (signed_header.validators_hash == validators.hash).true_or(Error::InvalidValidatorSet)
    }

    fn next_validators_match(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<(), Error> {
        (signed_header.validators_hash == validators.hash).true_or(Error::InvalidNextValidatorSet)
    }

    fn header_matches_commit(
        &self,
        header: &Header,
        commit: &Commit,
        header_hasher: impl HeaderHasher,
    ) -> Result<(), Error> {
        (header_hasher.hash(header) == commit.header_hash).true_or(Error::InvalidCommitValue)
    }

    fn valid_commit(
        &self,
        commit: &Commit,
        validators: &ValidatorSet,
        validator: impl CommitValidator,
    ) -> Result<(), Error> {
        validator.validate(commit, validators)
    }

    fn is_within_trust_period(
        &self,
        header: &Header,
        trusting_period: Duration,
        now: SystemTime,
    ) -> Result<(), Error> {
        let header_time: SystemTime = header.bft_time.into();
        let expires_at = header_time + trusting_period;

        (header_time < now && expires_at > now).true_or(Error::NotWithinTrustPeriod)
    }

    fn is_monotonic_bft_time(&self, header_a: &Header, header_b: &Header) -> Result<(), Error> {
        (header_b.bft_time >= header_a.bft_time).true_or(Error::NonMonotonicBftTime)
    }

    fn is_monotonic_height(&self, header_a: &Header, header_b: &Header) -> Result<(), Error> {
        (header_a.height > header_b.height).true_or(Error::NonIncreasingHeight)
    }

    fn has_sufficient_voting_power(
        &self,
        commit: &Commit,
        validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &impl VotingPowerCalculator,
    ) -> Result<(), Error> {
        let total_power = calculator.total_power_of(validators);
        let voting_power = calculator.voting_power_in(commit, validators);

        let result = if let (Ok(total_power), Ok(voting_power)) = (total_power, voting_power) {
            voting_power * trust_threshold.denominator > total_power * trust_threshold.numerator
        } else {
            false
        };

        result.true_or(Error::InsufficientVotingPower)
    }

    fn has_sufficient_validators_overlap(
        &self,
        untrusted_commit: &Commit,
        trusted_validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &impl VotingPowerCalculator,
    ) -> Result<(), Error> {
        self.has_sufficient_voting_power(
            untrusted_commit,
            trusted_validators,
            trust_threshold,
            calculator,
        )
        .map_err(|_| Error::InsufficientValidatorsOverlap)
    }

    fn has_sufficient_signers_overlap(
        &self,
        untrusted_commit: &Commit,
        untrusted_validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &impl VotingPowerCalculator,
    ) -> Result<(), Error> {
        self.has_sufficient_voting_power(
            untrusted_commit,
            untrusted_validators,
            trust_threshold,
            calculator,
        )
        .map_err(|_| Error::InvalidCommit)
    }

    fn valid_next_validator_set(
        &self,
        trusted_state: &TrustedState,
        untrusted_sh: &SignedHeader,
        untrusted_next_vals: &ValidatorSet,
    ) -> Result<(), Error> {
        let result = untrusted_sh.header.height == trusted_state.header.height
            && trusted_state.validators.hash != untrusted_next_vals.hash;

        result.false_or(Error::InvalidNextValidatorSet)
    }
}
