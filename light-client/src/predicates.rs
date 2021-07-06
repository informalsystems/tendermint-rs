//! Predicates for light block validation and verification.

use crate::{
    light_client::Options,
    operations::{CommitValidator, Hasher, VotingPowerCalculator},
    types::{Header, LightBlock, SignedHeader, Time, TrustThreshold, ValidatorSet},
};

use errors::{self as error, VerificationError};
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
pub trait VerificationPredicates: Send + Sync {
    /// Compare the provided validator_set_hash against the hash produced from hashing the validator
    /// set.
    fn validator_sets_match(
        &self,
        light_block: &LightBlock,
        hasher: &dyn Hasher,
    ) -> Result<(), VerificationError> {
        let validators_hash = hasher.hash_validator_set(&light_block.validators);

        if light_block.signed_header.header.validators_hash == validators_hash {
            Ok(())
        } else {
            Err(error::invalid_validator_set_error(
                light_block.signed_header.header.validators_hash,
                validators_hash,
            ))
        }
    }

    /// Check that the hash of the next validator set in the header match the actual one.
    fn next_validators_match(
        &self,
        light_block: &LightBlock,
        hasher: &dyn Hasher,
    ) -> Result<(), VerificationError> {
        let next_validators_hash = hasher.hash_validator_set(&light_block.next_validators);

        if light_block.signed_header.header.next_validators_hash == next_validators_hash {
            Ok(())
        } else {
            Err(error::invalid_next_validator_set_error(
                light_block.signed_header.header.next_validators_hash,
                next_validators_hash,
            ))
        }
    }

    /// Check that the hash of the header in the commit matches the actual one.
    fn header_matches_commit(
        &self,
        signed_header: &SignedHeader,
        hasher: &dyn Hasher,
    ) -> Result<(), VerificationError> {
        let header_hash = hasher.hash_header(&signed_header.header);

        if header_hash == signed_header.commit.block_id.hash {
            Ok(())
        } else {
            Err(error::invalid_commit_value_error(
                header_hash,
                signed_header.commit.block_id.hash,
            ))
        }
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

        if expires_at > now {
            Ok(())
        } else {
            Err(error::not_within_trust_period_error(expires_at, now))
        }
    }

    /// Check that the untrusted header is from past.
    fn is_header_from_past(
        &self,
        untrusted_header: &Header,
        clock_drift: Duration,
        now: Time,
    ) -> Result<(), VerificationError> {
        if untrusted_header.time < now + clock_drift {
            Ok(())
        } else {
            Err(error::header_from_the_future_error(
                untrusted_header.time,
                now,
            ))
        }
    }

    /// Check that time passed monotonically between the trusted header and the untrusted one.
    fn is_monotonic_bft_time(
        &self,
        untrusted_header: &Header,
        trusted_header: &Header,
    ) -> Result<(), VerificationError> {
        if untrusted_header.time > trusted_header.time {
            Ok(())
        } else {
            Err(error::non_monotonic_bft_time_error(
                untrusted_header.time,
                trusted_header.time,
            ))
        }
    }

    /// Check that the height increased between the trusted header and the untrusted one.
    fn is_monotonic_height(
        &self,
        untrusted_header: &Header,
        trusted_header: &Header,
    ) -> Result<(), VerificationError> {
        let trusted_height = trusted_header.height;

        if untrusted_header.height > trusted_header.height {
            Ok(())
        } else {
            Err(error::non_increasing_height_error(
                untrusted_header.height,
                trusted_height.increment(),
            ))
        }
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
        if light_block.signed_header.header.validators_hash
            == trusted_state.signed_header.header.next_validators_hash
        {
            Ok(())
        } else {
            Err(error::invalid_next_validator_set_error(
                light_block.signed_header.header.validators_hash,
                trusted_state.signed_header.header.next_validators_hash,
            ))
        }
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

#[cfg(test)]
mod tests {
    use flex_error::ErrorReport;
    use std::ops::Sub;
    use std::time::Duration;
    use tendermint::Time;

    use tendermint_testgen::{
        light_block::{LightBlock as TestgenLightBlock, TmLightBlock},
        Commit, Generator, Header, Validator, ValidatorSet,
    };

    use crate::predicates::{
        errors::VerificationErrorDetail, ProdPredicates, VerificationPredicates,
    };

    use crate::operations::{
        Hasher, ProdCommitValidator, ProdHasher, ProdVotingPowerCalculator, VotingPowerTally,
    };
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

        let vp = ProdPredicates::default();

        // 1. ensure valid header verifies
        let result_ok = vp.is_monotonic_bft_time(&header_two, &header_one);
        assert!(result_ok.is_ok());

        // 2. ensure header with non-monotonic bft time fails
        let result_err = vp.is_monotonic_bft_time(&header_one, &header_two);
        match result_err {
            Err(ErrorReport(VerificationErrorDetail::NonMonotonicBftTime(e), _)) => {
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

        let vp = ProdPredicates::default();

        // 1. ensure valid header verifies
        let result_ok = vp.is_monotonic_height(&header_two, &header_one);
        assert!(result_ok.is_ok());

        // 2. ensure header with non-monotonic height fails
        let result_err = vp.is_monotonic_height(&header_one, &header_two);

        match result_err {
            Err(ErrorReport(VerificationErrorDetail::NonIncreasingHeight(e), _)) => {
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

        let vp = ProdPredicates::default();

        // 1. ensure valid header verifies
        let mut trusting_period = Duration::new(1000, 0);
        let now = Time::now();

        let result_ok = vp.is_within_trust_period(&header, trusting_period, now);
        assert!(result_ok.is_ok());

        // 2. ensure header outside trusting period fails
        trusting_period = Duration::new(0, 1);

        let result_err = vp.is_within_trust_period(&header, trusting_period, now);

        let expires_at = header.time + trusting_period;
        match result_err {
            Err(ErrorReport(VerificationErrorDetail::NotWithinTrustPeriod(e), _)) => {
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

        let vp = ProdPredicates::default();
        let one_second = Duration::new(1, 0);

        // 1. ensure valid header verifies
        let result_ok = vp.is_header_from_past(&header, one_second, Time::now());

        assert!(result_ok.is_ok());

        // 2. ensure it fails if header is from a future time
        let now = Time::now().sub(one_second * 15);
        let result_err = vp.is_header_from_past(&header, one_second, now);

        match result_err {
            Err(ErrorReport(VerificationErrorDetail::HeaderFromTheFuture(e), _)) => {
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

        let vp = ProdPredicates::default();
        let hasher = ProdHasher::default();

        // Test positive case
        // 1. For predicate: validator_sets_match
        let val_sets_match_ok = vp.validator_sets_match(&light_block, &hasher);

        assert!(val_sets_match_ok.is_ok());

        // 2. For predicate: next_validator_sets_match
        let next_val_sets_match_ok = vp.next_validators_match(&light_block, &hasher);

        assert!(next_val_sets_match_ok.is_ok());

        // Test negative case
        // 1. For predicate: validator_sets_match
        light_block.validators = bad_validator_set.clone();

        let val_sets_match_err = vp.validator_sets_match(&light_block, &hasher);

        match val_sets_match_err {
            Err(ErrorReport(VerificationErrorDetail::InvalidValidatorSet(e), _)) => {
                assert_eq!(
                    e.header_validators_hash,
                    light_block.signed_header.header.validators_hash
                );
                assert_eq!(
                    e.validators_hash,
                    hasher.hash_validator_set(&light_block.validators)
                );
            }
            _ => panic!("expected InvalidValidatorSet error"),
        }

        // 2. For predicate: next_validator_sets_match
        light_block.next_validators = bad_validator_set;
        let next_val_sets_match_err = vp.next_validators_match(&light_block, &hasher);

        match next_val_sets_match_err {
            Err(ErrorReport(VerificationErrorDetail::InvalidNextValidatorSet(e), _)) => {
                assert_eq!(
                    e.header_next_validators_hash,
                    light_block.signed_header.header.next_validators_hash
                );
                assert_eq!(
                    e.next_validators_hash,
                    hasher.hash_validator_set(&light_block.next_validators)
                );
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

        let vp = ProdPredicates::default();
        let hasher = ProdHasher::default();

        // 1. ensure valid signed header verifies
        let result_ok = vp.header_matches_commit(&signed_header, &hasher);

        assert!(result_ok.is_ok());

        // 2. ensure invalid signed header fails
        signed_header.commit.block_id.hash =
            "15F15EF50BDE2018F4B129A827F90C18222C757770C8295EB8EE7BF50E761BC0"
                .parse()
                .unwrap();
        let result_err = vp.header_matches_commit(&signed_header, &hasher);

        // 3. ensure it fails with: VerificationError::InvalidCommitValue
        let header_hash = hasher.hash_header(&signed_header.header);

        match result_err {
            Err(ErrorReport(VerificationErrorDetail::InvalidCommitValue(e), _)) => {
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

        let vp = ProdPredicates::default();
        let hasher = ProdHasher::default();
        let commit_validator = ProdCommitValidator::new(hasher);

        // Test scenarios -->
        // 1. valid commit - must result "Ok"
        let mut result_ok = vp.valid_commit(&signed_header, &val_set, &commit_validator);

        assert!(result_ok.is_ok());

        // 2. no commit signatures - must return error
        let signatures = signed_header.commit.signatures.clone();
        signed_header.commit.signatures = vec![];

        let mut result_err = vp.valid_commit(&signed_header, &val_set, &commit_validator);

        match result_err {
            Err(ErrorReport(VerificationErrorDetail::NoSignatureForCommit(_), _)) => {}
            _ => panic!("expected ImplementationSpecific error"),
        }

        // 3. commit.signatures.len() != validator_set.validators().len()
        // must return error
        let mut bad_sigs = vec![signatures.clone().swap_remove(1)];
        signed_header.commit.signatures = bad_sigs.clone();

        result_err = vp.valid_commit(&signed_header, &val_set, &commit_validator);

        match result_err {
            Err(ErrorReport(VerificationErrorDetail::MismatchPreCommitLength(e), _)) => {
                assert_eq!(e.pre_commit_length, signed_header.commit.signatures.len());
                assert_eq!(e.validator_length, val_set.validators().len());
            }
            _ => panic!("expected ImplementationSpecific error"),
        }

        // 4. commit.BlockIdFlagAbsent - should be "Ok"
        bad_sigs.push(CommitSig::BlockIdFlagAbsent);
        signed_header.commit.signatures = bad_sigs;
        result_ok = vp.valid_commit(&signed_header, &val_set, &commit_validator);
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

        result_err = vp.valid_commit(
            &signed_header,
            &val_set_with_faulty_signer,
            &commit_validator,
        );

        match result_err {
            Err(ErrorReport(VerificationErrorDetail::FaultySigner(e), _)) => {
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

                assert_eq!(
                    e.validator_set,
                    hasher.hash_validator_set(&val_set_with_faulty_signer)
                );
            }
            _ => panic!("expected ImplementationSpecific error"),
        }
    }

    #[test]
    fn test_valid_next_validator_set() {
        let test_lb1 = TestgenLightBlock::new_default(1);
        let light_block1: LightBlock = test_lb1.generate().unwrap().into();

        let light_block2: LightBlock = test_lb1.next().generate().unwrap().into();

        let vp = ProdPredicates::default();

        // Test scenarios -->
        // 1. next_validator_set hash matches
        let result_ok = vp.valid_next_validator_set(&light_block1, &light_block2);

        assert!(result_ok.is_ok());

        // 2. next_validator_set hash doesn't match
        let vals = &[Validator::new("new-1"), Validator::new("new-2")];
        let header = Header::new(vals);
        let commit = Commit::new(header.clone(), 1);

        let light_block3: LightBlock = TestgenLightBlock::new(header, commit)
            .generate()
            .unwrap()
            .into();

        let result_err = vp.valid_next_validator_set(&light_block3, &light_block2);

        match result_err {
            Err(ErrorReport(VerificationErrorDetail::InvalidNextValidatorSet(e), _)) => {
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

        let vp = ProdPredicates::default();
        let mut trust_threshold = TrustThreshold::new(1, 3).expect("Cannot make trust threshold");
        let voting_power_calculator = ProdVotingPowerCalculator::default();

        // Test scenarios -->
        // 1. > trust_threshold validators overlap
        let result_ok = vp.has_sufficient_validators_overlap(
            &signed_header,
            &val_set,
            &trust_threshold,
            &voting_power_calculator,
        );

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

        let result_err = vp.has_sufficient_validators_overlap(
            &signed_header,
            &bad_valset,
            &trust_threshold,
            &voting_power_calculator,
        );

        match result_err {
            Err(ErrorReport(VerificationErrorDetail::NotEnoughTrust(e), _)) => {
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

        let vp = ProdPredicates::default();
        let voting_power_calculator = ProdVotingPowerCalculator::default();

        // Test scenarios -->
        // 1. +2/3 validators sign
        let result_ok = vp.has_sufficient_signers_overlap(
            &light_block.signed_header,
            &light_block.validators,
            &voting_power_calculator,
        );

        assert!(result_ok.is_ok());

        // 1. less than 2/3 validators sign
        light_block.signed_header.commit.signatures.pop();

        let result_err = vp.has_sufficient_signers_overlap(
            &light_block.signed_header,
            &light_block.validators,
            &voting_power_calculator,
        );

        let trust_threshold = TrustThreshold::TWO_THIRDS;

        match result_err {
            Err(ErrorReport(VerificationErrorDetail::InsufficientSignersOverlap(e), _)) => {
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
