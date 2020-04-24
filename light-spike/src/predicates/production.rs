use super::VerificationPredicates;
use crate::prelude::*;

pub struct ProductionPredicates;

impl VerificationPredicates for ProductionPredicates {
    fn validator_sets_match(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<(), Error> {
        (signed_header.validators_hash == validators.hash).true_or(Error::InvalidValidatorSet {
            header_validators_hash: signed_header.validators_hash,
            validators_hash: validators.hash,
        })
    }

    fn next_validators_match(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<(), Error> {
        (signed_header.validators_hash == validators.hash).true_or(Error::InvalidNextValidatorSet {
            header_next_validators_hash: signed_header.validators_hash,
            next_validators_hash: validators.hash,
        })
    }

    fn header_matches_commit(
        &self,
        header: &Header,
        commit: &Commit,
        header_hasher: impl HeaderHasher,
    ) -> Result<(), Error> {
        let header_hash = header_hasher.hash(header);
        (header_hash == commit.header_hash).true_or(Error::InvalidCommitValue {
            header_hash,
            commit_hash: commit.header_hash,
        })
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

        (header_time < now && expires_at > now).true_or(Error::NotWithinTrustPeriod {
            at: expires_at,
            now,
        })?;

        (header_time <= now).true_or(Error::HeaderFromTheFuture { header_time, now })?;

        Ok(())
    }

    fn is_monotonic_bft_time(
        &self,
        untrusted_header: &Header,
        trusted_header: &Header,
    ) -> Result<(), Error> {
        (untrusted_header.bft_time > trusted_header.bft_time).true_or(Error::NonMonotonicBftTime {
            header_bft_time: untrusted_header.bft_time,
            trusted_header_bft_time: trusted_header.bft_time,
        })
    }

    fn is_monotonic_height(
        &self,
        untrusted_header: &Header,
        trusted_header: &Header,
    ) -> Result<(), Error> {
        (untrusted_header.height > trusted_header.height).true_or(Error::NonIncreasingHeight {
            got: untrusted_header.height,
            expected: trusted_header.height + 1,
        })
    }

    fn has_sufficient_voting_power(
        &self,
        commit: &Commit,
        validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &impl VotingPowerCalculator,
    ) -> Result<(), (Error, u64, u64)> {
        let total_power = calculator.total_power_of(validators);
        let voting_power = calculator.voting_power_in(commit, validators);

        let result =
            voting_power * trust_threshold.denominator > total_power * trust_threshold.numerator;

        result.true_or((
            Error::InsufficientVotingPower {
                total_power,
                voting_power,
            },
            total_power,
            voting_power,
        ))
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
        .map_err(
            |(_, total_power, signed_power)| Error::InsufficientValidatorsOverlap {
                total_power,
                signed_power,
            },
        )
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
        .map_err(|(_, total_power, signed_power)| Error::InvalidCommit {
            total_power,
            signed_power,
        })
    }

    fn valid_next_validator_set(
        &self,
        untrusted_sh: &SignedHeader,
        untrusted_next_vals: &ValidatorSet,
    ) -> Result<(), Error> {
        (untrusted_sh.header.next_validators_hash != untrusted_next_vals.hash).false_or(
            Error::InvalidNextValidatorSet {
                header_next_validators_hash: untrusted_next_vals.hash,
                next_validators_hash: untrusted_next_vals.hash,
            },
        )
    }
}
