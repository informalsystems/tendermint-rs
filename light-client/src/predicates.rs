//! Predicates for light block validation and verification.

use crate::{
    light_client::Options,
    operations::{CommitValidator, VotingPowerCalculator},
    types::{Header, LightBlock, SignedHeader, Time, ValidatorSet},
};

use errors::VerificationError;
use std::time::Duration;

pub mod errors;

/// Compare the provided validator_set_hash against the hash produced from
/// hashing the validator set.
pub fn validator_sets_match(light_block: &LightBlock) -> Result<(), VerificationError> {
    let validators_hash = light_block.validators.hash();

    if light_block.signed_header.header.validators_hash == validators_hash {
        Ok(())
    } else {
        Err(VerificationError::invalid_validator_set(
            light_block.signed_header.header.validators_hash,
            validators_hash,
        ))
    }
}

/// Check that the hash of the next validator set in the header match the actual
/// one.
pub fn next_validators_match(light_block: &LightBlock) -> Result<(), VerificationError> {
    let next_validators_hash = light_block.next_validators.hash();

    if light_block.signed_header.header.next_validators_hash == next_validators_hash {
        Ok(())
    } else {
        Err(VerificationError::invalid_next_validator_set(
            light_block.signed_header.header.next_validators_hash,
            next_validators_hash,
        ))
    }
}

/// Check that the hash of the header in the commit matches the actual one.
pub fn header_matches_commit(signed_header: &SignedHeader) -> Result<(), VerificationError> {
    let header_hash = signed_header.header.hash();

    if header_hash == signed_header.commit.block_id.hash {
        Ok(())
    } else {
        Err(VerificationError::invalid_commit_value(
            header_hash,
            signed_header.commit.block_id.hash,
        ))
    }
}

/// Validate the commit using the given commit validator.
pub fn validate_commit<C: CommitValidator>(
    signed_header: &SignedHeader,
    validators: &ValidatorSet,
    commit_validator: &C,
) -> Result<(), VerificationError> {
    commit_validator.validate(signed_header, validators)?;
    commit_validator.validate_full(signed_header, validators)?;

    Ok(())
}

/// Check that the trusted header is within the trusting period, adjusting for clock drift.
pub fn is_within_trust_period(
    trusted_header: &Header,
    trusting_period: Duration,
    now: Time,
) -> Result<(), VerificationError> {
    let expires_at = trusted_header.time + trusting_period;

    if expires_at > now {
        Ok(())
    } else {
        Err(VerificationError::not_within_trust_period(expires_at, now))
    }
}

/// Check that the untrusted header is from past.
pub fn is_header_from_past(
    untrusted_header: &Header,
    clock_drift: Duration,
    now: Time,
) -> Result<(), VerificationError> {
    if untrusted_header.time < now + clock_drift {
        Ok(())
    } else {
        Err(VerificationError::header_from_the_future(
            untrusted_header.time,
            now,
        ))
    }
}

/// Check that time passed monotonically between the trusted header and the
/// untrusted one.
pub fn is_monotonic_bft_time(
    untrusted_header: &Header,
    trusted_header: &Header,
) -> Result<(), VerificationError> {
    if untrusted_header.time > trusted_header.time {
        Ok(())
    } else {
        Err(VerificationError::non_monotonic_bft_time(
            untrusted_header.time,
            trusted_header.time,
        ))
    }
}

/// Check that the height increased between the trusted header and the untrusted
/// one.
pub fn is_monotonic_height(
    untrusted_header: &Header,
    trusted_header: &Header,
) -> Result<(), VerificationError> {
    let trusted_height = trusted_header.height;

    if untrusted_header.height > trusted_header.height {
        Ok(())
    } else {
        Err(VerificationError::non_increasing_height(
            untrusted_header.height,
            trusted_height.increment(),
        ))
    }
}

/// Check that the hash of the next validator set in the trusted block matches
/// the hash of the validator set in the untrusted one.
pub fn valid_next_validator_set(
    light_block: &LightBlock,
    trusted_state: &LightBlock,
) -> Result<(), VerificationError> {
    if light_block.signed_header.header.validators_hash
        == trusted_state.signed_header.header.next_validators_hash
    {
        Ok(())
    } else {
        Err(VerificationError::invalid_next_validator_set(
            light_block.signed_header.header.validators_hash,
            trusted_state.signed_header.header.next_validators_hash,
        ))
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
pub fn verify<C, V>(
    voting_power_calculator: &C,
    commit_validator: &V,
    trusted: &LightBlock,
    untrusted: &LightBlock,
    options: &Options,
    now: Time,
) -> Result<(), VerificationError>
where
    C: VotingPowerCalculator,
    V: CommitValidator,
{
    // Ensure the latest trusted header hasn't expired
    is_within_trust_period(&trusted.signed_header.header, options.trusting_period, now)?;

    // Ensure the header isn't from a future time
    is_header_from_past(&untrusted.signed_header.header, options.clock_drift, now)?;

    // Ensure the header validator hashes match the given validators
    validator_sets_match(untrusted)?;

    // Ensure the header next validator hashes match the given next validators
    next_validators_match(untrusted)?;

    // Ensure the header matches the commit
    header_matches_commit(&untrusted.signed_header)?;

    // Additional implementation specific validation
    validate_commit(
        &untrusted.signed_header,
        &untrusted.validators,
        commit_validator,
    )?;

    // Check that the untrusted block is more recent than the trusted state
    is_monotonic_bft_time(
        &untrusted.signed_header.header,
        &trusted.signed_header.header,
    )?;

    let trusted_next_height = trusted.height().increment();

    if untrusted.height() == trusted_next_height {
        // If the untrusted block is the very next block after the trusted block,
        // check that their (next) validator sets hashes match.
        valid_next_validator_set(untrusted, trusted)?;
    } else {
        // Otherwise, ensure that the untrusted block has a greater height than
        // the trusted block.
        is_monotonic_height(
            &untrusted.signed_header.header,
            &trusted.signed_header.header,
        )?;

        // Check there is enough overlap between the validator sets of
        // the trusted and untrusted blocks.
        voting_power_calculator.check_enough_trust(
            &untrusted.signed_header,
            &trusted.next_validators,
            options.trust_threshold,
        )?;
    }

    // Verify that more than 2/3 of the validators correctly committed the block.
    voting_power_calculator
        .check_signers_overlap(&untrusted.signed_header, &untrusted.validators)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ops::Sub;
    use std::time::Duration;
    use tendermint::Time;

    use tendermint_testgen::{
        light_block::{LightBlock as TestgenLightBlock, TmLightBlock},
        Commit, Generator, Header, Validator, ValidatorSet,
    };

    use crate::predicates::errors::{VerificationError, VerificationErrorDetail};

    use crate::operations::{ProdCommitValidator, ProdVotingPowerCalculator, VotingPowerTally};
    use crate::types::{LightBlock, TrustThreshold};
    use tendermint::block::CommitSig;
    use tendermint::validator::Set;

    impl From<TmLightBlock> for LightBlock {
        fn from(lb: TmLightBlock) -> Self {
            LightBlock {
                signed_header: lb.signed_header,
                validators: lb.validators,
                next_validators: lb.next_validators,
                provider: lb.provider,
            }
        }
    }

    #[test]
    fn test_is_monotonic_bft_time() {
        let val = vec![Validator::new("val-1")];
        let header_one = Header::new(&val).generate().unwrap();
        let header_two = Header::new(&val).generate().unwrap();

        // 1. ensure valid header verifies
        let result_ok = is_monotonic_bft_time(&header_two, &header_one);
        assert!(result_ok.is_ok());

        // 2. ensure header with non-monotonic bft time fails
        let result_err = is_monotonic_bft_time(&header_one, &header_two);
        match result_err {
            Err(VerificationError(VerificationErrorDetail::NonMonotonicBftTime(e), _)) => {
                assert_eq!(e.header_bft_time, header_one.time);
                assert_eq!(e.trusted_header_bft_time, header_two.time);
            }
            _ => panic!("expected NonMonotonicBftTime error"),
        }
    }

    #[test]
    fn test_is_monotonic_height() {
        let val = vec![Validator::new("val-1")];
        let header_one = Header::new(&val).generate().unwrap();
        let header_two = Header::new(&val).height(2).generate().unwrap();

        // 1. ensure valid header verifies
        let result_ok = is_monotonic_height(&header_two, &header_one);
        assert!(result_ok.is_ok());

        // 2. ensure header with non-monotonic height fails
        let result_err = is_monotonic_height(&header_one, &header_two);

        match result_err {
            Err(VerificationError(VerificationErrorDetail::NonIncreasingHeight(e), _)) => {
                assert_eq!(e.got, header_one.height);
                assert_eq!(e.expected, header_two.height.increment());
            }
            _ => panic!("expected NonIncreasingHeight error"),
        }
    }

    #[test]
    fn test_is_within_trust_period() {
        let val = Validator::new("val-1");
        let header = Header::new(&[val]).generate().unwrap();

        // 1. ensure valid header verifies
        let mut trusting_period = Duration::new(1000, 0);
        let now = Time::now();

        let result_ok = is_within_trust_period(&header, trusting_period, now);
        assert!(result_ok.is_ok());

        // 2. ensure header outside trusting period fails
        trusting_period = Duration::new(0, 1);

        let result_err = is_within_trust_period(&header, trusting_period, now);

        let expires_at = header.time + trusting_period;
        match result_err {
            Err(VerificationError(VerificationErrorDetail::NotWithinTrustPeriod(e), _)) => {
                assert_eq!(e.expires_at, expires_at);
                assert_eq!(e.now, now);
            }
            _ => panic!("expected NotWithinTrustPeriod error"),
        }
    }

    #[test]
    fn test_is_header_from_past() {
        let val = Validator::new("val-1");
        let header = Header::new(&[val]).generate().unwrap();

        let one_second = Duration::new(1, 0);

        // 1. ensure valid header verifies
        let result_ok = is_header_from_past(&header, one_second, Time::now());

        assert!(result_ok.is_ok());

        // 2. ensure it fails if header is from a future time
        let now = Time::now().sub(one_second * 15);
        let result_err = is_header_from_past(&header, one_second, now);

        match result_err {
            Err(VerificationError(VerificationErrorDetail::HeaderFromTheFuture(e), _)) => {
                assert_eq!(e.header_time, header.time);
                assert_eq!(e.now, now);
            }
            _ => panic!("expected HeaderFromTheFuture error"),
        }
    }

    #[test]
    // NOTE: tests both current valset and next valset
    fn test_validator_sets_match() {
        let mut light_block: LightBlock =
            TestgenLightBlock::new_default(1).generate().unwrap().into();

        let bad_validator_set = ValidatorSet::new(vec!["bad-val"]).generate().unwrap();

        // Test positive case
        // 1. For predicate: validator_sets_match
        let val_sets_match_ok = validator_sets_match(&light_block);

        assert!(val_sets_match_ok.is_ok());

        // 2. For predicate: next_validator_sets_match
        let next_val_sets_match_ok = next_validators_match(&light_block);

        assert!(next_val_sets_match_ok.is_ok());

        // Test negative case
        // 1. For predicate: validator_sets_match
        light_block.validators = bad_validator_set.clone();

        let val_sets_match_err = validator_sets_match(&light_block);

        match val_sets_match_err {
            Err(VerificationError(VerificationErrorDetail::InvalidValidatorSet(e), _)) => {
                assert_eq!(
                    e.header_validators_hash,
                    light_block.signed_header.header.validators_hash
                );
                assert_eq!(e.validators_hash, light_block.validators.hash());
            }
            _ => panic!("expected InvalidValidatorSet error"),
        }

        // 2. For predicate: next_validator_sets_match
        light_block.next_validators = bad_validator_set;
        let next_val_sets_match_err = next_validators_match(&light_block);

        match next_val_sets_match_err {
            Err(VerificationError(VerificationErrorDetail::InvalidNextValidatorSet(e), _)) => {
                assert_eq!(
                    e.header_next_validators_hash,
                    light_block.signed_header.header.next_validators_hash
                );
                assert_eq!(e.next_validators_hash, light_block.next_validators.hash());
            }
            _ => panic!("expected InvalidNextValidatorSet error"),
        }
    }

    #[test]
    fn test_header_matches_commit() {
        let mut signed_header = TestgenLightBlock::new_default(1)
            .generate()
            .unwrap()
            .signed_header;

        // 1. ensure valid signed header verifies
        let result_ok = header_matches_commit(&signed_header);

        assert!(result_ok.is_ok());

        // 2. ensure invalid signed header fails
        signed_header.commit.block_id.hash =
            "15F15EF50BDE2018F4B129A827F90C18222C757770C8295EB8EE7BF50E761BC0"
                .parse()
                .unwrap();
        let result_err = header_matches_commit(&signed_header);

        // 3. ensure it fails with: VerificationVerificationError::InvalidCommitValue
        let header_hash = signed_header.header.hash();

        match result_err {
            Err(VerificationError(VerificationErrorDetail::InvalidCommitValue(e), _)) => {
                assert_eq!(e.header_hash, header_hash);
                assert_eq!(e.commit_hash, signed_header.commit.block_id.hash);
            }
            _ => panic!("expected InvalidCommitValue error"),
        }
    }

    #[test]
    fn test_valid_commit() {
        let light_block: LightBlock = TestgenLightBlock::new_default(1).generate().unwrap().into();

        let mut signed_header = light_block.signed_header;
        let val_set = light_block.validators;

        let commit_validator = ProdCommitValidator;

        // Test scenarios -->
        // 1. valid commit - must result "Ok"
        let mut result_ok = validate_commit(&signed_header, &val_set, &commit_validator);

        assert!(result_ok.is_ok());

        // 2. no commit signatures - must return error
        let signatures = signed_header.commit.signatures.clone();
        signed_header.commit.signatures = vec![];

        let mut result_err = validate_commit(&signed_header, &val_set, &commit_validator);

        match result_err {
            Err(VerificationError(VerificationErrorDetail::NoSignatureForCommit(_), _)) => {}
            _ => panic!("expected ImplementationSpecific error"),
        }

        // 3. commit.signatures.len() != validator_set.validators().len()
        // must return error
        let mut bad_sigs = vec![signatures.clone().swap_remove(1)];
        signed_header.commit.signatures = bad_sigs.clone();

        result_err = validate_commit(&signed_header, &val_set, &commit_validator);

        match result_err {
            Err(VerificationError(VerificationErrorDetail::MismatchPreCommitLength(e), _)) => {
                assert_eq!(e.pre_commit_length, signed_header.commit.signatures.len());
                assert_eq!(e.validator_length, val_set.validators().len());
            }
            _ => panic!("expected MismatchPreCommitLength error"),
        }

        // 4. commit.BlockIdFlagAbsent - should be "Ok"
        bad_sigs.push(CommitSig::BlockIdFlagAbsent);
        signed_header.commit.signatures = bad_sigs;
        result_ok = validate_commit(&signed_header, &val_set, &commit_validator);
        assert!(result_ok.is_ok());

        // 5. faulty signer - must return error
        let mut bad_vals = val_set.validators().clone();
        bad_vals.pop();
        bad_vals.push(
            Validator::new("bad-val")
                .generate()
                .expect("Failed to generate validator"),
        );
        let val_set_with_faulty_signer = Set::without_proposer(bad_vals);

        // reset signatures
        signed_header.commit.signatures = signatures;

        result_err = validate_commit(
            &signed_header,
            &val_set_with_faulty_signer,
            &commit_validator,
        );

        match result_err {
            Err(VerificationError(VerificationErrorDetail::FaultySigner(e), _)) => {
                assert_eq!(
                    e.signer,
                    signed_header
                        .commit
                        .signatures
                        .iter()
                        .last()
                        .unwrap()
                        .validator_address()
                        .unwrap()
                );

                assert_eq!(e.validator_set, val_set_with_faulty_signer.hash());
            }
            _ => panic!("expected FaultySigner error"),
        }
    }

    #[test]
    fn test_valid_next_validator_set() {
        let test_lb1 = TestgenLightBlock::new_default(1);
        let light_block1: LightBlock = test_lb1.generate().unwrap().into();

        let light_block2: LightBlock = test_lb1.next().generate().unwrap().into();

        // Test scenarios -->
        // 1. next_validator_set hash matches
        let result_ok = valid_next_validator_set(&light_block1, &light_block2);

        assert!(result_ok.is_ok());

        // 2. next_validator_set hash doesn't match
        let vals = &[Validator::new("new-1"), Validator::new("new-2")];
        let header = Header::new(vals);
        let commit = Commit::new(header.clone(), 1);

        let light_block3: LightBlock = TestgenLightBlock::new(header, commit)
            .generate()
            .unwrap()
            .into();

        let result_err = valid_next_validator_set(&light_block3, &light_block2);

        match result_err {
            Err(VerificationError(VerificationErrorDetail::InvalidNextValidatorSet(e), _)) => {
                assert_eq!(
                    e.header_next_validators_hash,
                    light_block3.signed_header.header.validators_hash
                );
                assert_eq!(
                    e.next_validators_hash,
                    light_block2.signed_header.header.next_validators_hash
                );
            }
            _ => panic!("expected InvalidNextValidatorSet error"),
        }
    }

    #[test]
    fn test_has_sufficient_validators_overlap() {
        let light_block: LightBlock = TestgenLightBlock::new_default(1).generate().unwrap().into();
        let val_set = light_block.validators;
        let signed_header = light_block.signed_header;

        let mut trust_threshold = TrustThreshold::new(1, 3).expect("Cannot make trust threshold");
        let voting_power_calculator = ProdVotingPowerCalculator::default();

        // Test scenarios -->
        // 1. > trust_threshold validators overlap
        let result_ok =
            voting_power_calculator.check_enough_trust(&signed_header, &val_set, trust_threshold);

        assert!(result_ok.is_ok());

        // 2. < trust_threshold validators overlap
        let mut vals = val_set.validators().clone();
        vals.push(
            Validator::new("extra-val")
                .voting_power(100)
                .generate()
                .unwrap(),
        );
        let bad_valset = Set::without_proposer(vals);

        trust_threshold = TrustThreshold::new(2, 3).expect("Cannot make trust threshold");

        let result_err = voting_power_calculator.check_enough_trust(
            &signed_header,
            &bad_valset,
            trust_threshold,
        );

        match result_err {
            Err(VerificationError(VerificationErrorDetail::NotEnoughTrust(e), _)) => {
                assert_eq!(
                    e.tally,
                    VotingPowerTally {
                        total: 200,
                        tallied: 100,
                        trust_threshold,
                    }
                );
            }
            _ => panic!("expected NotEnoughTrust error"),
        }
    }

    #[test]
    fn test_has_sufficient_signers_overlap() {
        let mut light_block: LightBlock =
            TestgenLightBlock::new_default(2).generate().unwrap().into();

        let voting_power_calculator = ProdVotingPowerCalculator::default();

        // Test scenarios -->
        // 1. +2/3 validators sign
        let result_ok = voting_power_calculator
            .check_signers_overlap(&light_block.signed_header, &light_block.validators);

        assert!(result_ok.is_ok());

        // 1. less than 2/3 validators sign
        light_block.signed_header.commit.signatures.pop();

        let result_err = voting_power_calculator
            .check_signers_overlap(&light_block.signed_header, &light_block.validators);

        let trust_threshold = TrustThreshold::TWO_THIRDS;

        match result_err {
            Err(VerificationError(VerificationErrorDetail::InsufficientSignersOverlap(e), _)) => {
                assert_eq!(
                    e.tally,
                    VotingPowerTally {
                        total: 100,
                        tallied: 50,
                        trust_threshold,
                    }
                );
            }
            _ => panic!("expected InsufficientSignersOverlap error"),
        }
    }
}
