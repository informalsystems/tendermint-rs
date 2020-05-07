use crate::prelude::*;

use tendermint::lite::types::Commit as _;
use tendermint::lite::ValidatorSet as _;

#[derive(Copy, Clone, Debug)]
pub struct ProductionPredicates;

impl VerificationPredicates for ProductionPredicates {
    fn validator_sets_match(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<(), VerificationError> {
        ensure!(
            signed_header.header.validators_hash == validators.hash(),
            VerificationError::InvalidValidatorSet {
                header_validators_hash: signed_header.header.validators_hash,
                validators_hash: validators.hash(),
            }
        );

        Ok(())
    }

    fn next_validators_match(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<(), VerificationError> {
        ensure!(
            signed_header.header.validators_hash == validators.hash(),
            VerificationError::InvalidNextValidatorSet {
                header_next_validators_hash: signed_header.header.validators_hash,
                next_validators_hash: validators.hash(),
            }
        );

        Ok(())
    }

    fn header_matches_commit(
        &self,
        header: &Header,
        signed_header: &SignedHeader,
        header_hasher: &dyn HeaderHasher,
    ) -> Result<(), VerificationError> {
        let header_hash = header_hasher.hash(header);

        ensure!(
            header_hash == signed_header.header_hash(),
            VerificationError::InvalidCommitValue {
                header_hash,
                commit_hash: signed_header.header_hash(),
            }
        );

        Ok(())
    }

    fn valid_commit(
        &self,
        commit: &Commit,
        validators: &ValidatorSet,
        validator: &dyn CommitValidator,
    ) -> Result<(), VerificationError> {
        // FIXME: Do not discard underlying error
        validator
            .validate(commit, validators)
            .map_err(|e| VerificationError::InvalidCommit(e.to_string()))?;

        Ok(())
    }

    fn is_within_trust_period(
        &self,
        header: &Header,
        trusting_period: Duration,
        now: Time,
    ) -> Result<(), VerificationError> {
        let expires_at = header.time + trusting_period;

        ensure!(
            header.time < now && expires_at > now,
            VerificationError::NotWithinTrustPeriod {
                at: expires_at,
                now,
            }
        );

        ensure!(
            header.time <= now,
            VerificationError::HeaderFromTheFuture {
                header_time: header.time,
                now
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
                voting_power: Some(voting_power),
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
        self.has_sufficient_voting_power(
            untrusted_sh,
            trusted_validators,
            trust_threshold,
            calculator,
        )
        .map_err(|_| {
            let total_power = calculator.total_power_of(trusted_validators);
            let signed_power = calculator
                .voting_power_in(untrusted_sh, trusted_validators)
                .ok();

            VerificationError::InsufficientValidatorsOverlap {
                total_power,
                signed_power,
            }
        })
    }

    fn has_sufficient_signers_overlap(
        &self,
        untrusted_sh: &SignedHeader,
        untrusted_validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError> {
        self.has_sufficient_voting_power(
            untrusted_sh,
            untrusted_validators,
            trust_threshold,
            calculator,
        )
        .map_err(|_| {
            let total_power = calculator.total_power_of(untrusted_validators);
            let signed_power = calculator
                .voting_power_in(untrusted_sh, untrusted_validators)
                .ok();

            VerificationError::InsufficientCommitPower {
                total_power,
                signed_power,
            }
        })
    }

    fn valid_next_validator_set(
        &self,
        untrusted_sh: &SignedHeader,
        untrusted_next_vals: &ValidatorSet,
    ) -> Result<(), VerificationError> {
        ensure!(
            untrusted_sh.header.next_validators_hash == untrusted_next_vals.hash(),
            VerificationError::InvalidNextValidatorSet {
                header_next_validators_hash: untrusted_next_vals.hash(),
                next_validators_hash: untrusted_next_vals.hash(),
            }
        );

        Ok(())
    }
}
