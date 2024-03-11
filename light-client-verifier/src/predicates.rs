//! Predicates for light block validation and verification.

use core::time::Duration;

use tendermint::{
    block::Height, chain::Id as ChainId, crypto::Sha256, hash::Hash, merkle::MerkleHash,
};

use crate::{
    errors::VerificationError,
    operations::{CommitValidator, VotingPowerCalculator},
    prelude::*,
    types::{Header, SignedHeader, Time, TrustThreshold, ValidatorSet},
};

/// Production predicates, using the default implementation
/// of the `VerificationPredicates` trait.
#[cfg(feature = "rust-crypto")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProdPredicates;

#[cfg(feature = "rust-crypto")]
impl VerificationPredicates for ProdPredicates {
    type Sha256 = tendermint::crypto::default::Sha256;
}

/// Defines the various predicates used to validate and verify light blocks.
///
/// A default, spec abiding implementation is provided for each method.
///
/// This enables test implementations to only override a single method rather than
/// have to re-define every predicate.
pub trait VerificationPredicates: Send + Sync {
    /// The implementation of SHA256 digest
    type Sha256: MerkleHash + Sha256 + Default;

    /// Compare the provided validator_set_hash against the hash produced from hashing the validator
    /// set.
    fn validator_sets_match(
        &self,
        validators: &ValidatorSet,
        header_validators_hash: Hash,
    ) -> Result<(), VerificationError> {
        let validators_hash = validators.hash_with::<Self::Sha256>();
        if header_validators_hash == validators_hash {
            Ok(())
        } else {
            Err(VerificationError::invalid_validator_set(
                header_validators_hash,
                validators_hash,
            ))
        }
    }

    /// Check that the hash of the next validator set in the header match the actual one.
    fn next_validators_match(
        &self,
        next_validators: &ValidatorSet,
        header_next_validators_hash: Hash,
    ) -> Result<(), VerificationError> {
        let next_validators_hash = next_validators.hash_with::<Self::Sha256>();
        if header_next_validators_hash == next_validators_hash {
            Ok(())
        } else {
            Err(VerificationError::invalid_next_validator_set(
                header_next_validators_hash,
                next_validators_hash,
            ))
        }
    }

    /// Check that the hash of the header in the commit matches the actual one.
    fn header_matches_commit(
        &self,
        header: &Header,
        commit_hash: Hash,
    ) -> Result<(), VerificationError> {
        let header_hash = header.hash_with::<Self::Sha256>();
        if header_hash == commit_hash {
            Ok(())
        } else {
            Err(VerificationError::invalid_commit_value(
                header_hash,
                commit_hash,
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
        trusted_header_time: Time,
        trusting_period: Duration,
        now: Time,
    ) -> Result<(), VerificationError> {
        let expires_at =
            (trusted_header_time + trusting_period).map_err(VerificationError::tendermint)?;

        if expires_at > now {
            Ok(())
        } else {
            Err(VerificationError::not_within_trust_period(expires_at, now))
        }
    }

    /// Check that the untrusted header is from past.
    fn is_header_from_past(
        &self,
        untrusted_header_time: Time,
        clock_drift: Duration,
        now: Time,
    ) -> Result<(), VerificationError> {
        let drifted = (now + clock_drift).map_err(VerificationError::tendermint)?;

        if untrusted_header_time < drifted {
            Ok(())
        } else {
            Err(VerificationError::header_from_the_future(
                untrusted_header_time,
                now,
                clock_drift,
            ))
        }
    }

    /// Check that time passed monotonically between the trusted header and the untrusted one.
    fn is_monotonic_bft_time(
        &self,
        untrusted_header_time: Time,
        trusted_header_time: Time,
    ) -> Result<(), VerificationError> {
        if untrusted_header_time > trusted_header_time {
            Ok(())
        } else {
            Err(VerificationError::non_monotonic_bft_time(
                untrusted_header_time,
                trusted_header_time,
            ))
        }
    }

    /// Check that the height increased between the trusted header and the untrusted one.
    fn is_monotonic_height(
        &self,
        untrusted_height: Height,
        trusted_height: Height,
    ) -> Result<(), VerificationError> {
        if untrusted_height > trusted_height {
            Ok(())
        } else {
            Err(VerificationError::non_increasing_height(
                untrusted_height,
                trusted_height.increment(),
            ))
        }
    }

    /// Check that the chain-ids of the trusted header and the untrusted one are the same
    fn is_matching_chain_id(
        &self,
        untrusted_chain_id: &ChainId,
        trusted_chain_id: &ChainId,
    ) -> Result<(), VerificationError> {
        if untrusted_chain_id == trusted_chain_id {
            Ok(())
        } else {
            Err(VerificationError::chain_id_mismatch(
                untrusted_chain_id.to_string(),
                trusted_chain_id.to_string(),
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
        untrusted_validators_hash: Hash,
        trusted_next_validators_hash: Hash,
    ) -> Result<(), VerificationError> {
        if trusted_next_validators_hash == untrusted_validators_hash {
            Ok(())
        } else {
            Err(VerificationError::invalid_next_validator_set(
                untrusted_validators_hash,
                trusted_next_validators_hash,
            ))
        }
    }
}

#[cfg(all(test, feature = "rust-crypto"))]
mod tests {
    use core::time::Duration;

    use tendermint::{block::CommitSig, validator::Set};
    use tendermint_testgen::{
        light_block::{LightBlock as TestgenLightBlock, TmLightBlock},
        Commit, Generator, Header, Validator, ValidatorSet,
    };
    use time::OffsetDateTime;

    use crate::{
        errors::{VerificationError, VerificationErrorDetail},
        operations::{ProdCommitValidator, ProdVotingPowerCalculator, VotingPowerTally},
        predicates::{ProdPredicates, VerificationPredicates},
        prelude::*,
        types::{LightBlock, TrustThreshold},
    };

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

        let vp = ProdPredicates;

        // 1. ensure valid header verifies
        let result_ok = vp.is_monotonic_bft_time(header_two.time, header_one.time);
        assert!(result_ok.is_ok());

        // 2. ensure header with non-monotonic bft time fails
        let result_err = vp.is_monotonic_bft_time(header_one.time, header_two.time);
        match result_err {
            Err(VerificationError(VerificationErrorDetail::NonMonotonicBftTime(e), _)) => {
                assert_eq!(e.header_bft_time, header_one.time);
                assert_eq!(e.trusted_header_bft_time, header_two.time);
            },
            _ => panic!("expected NonMonotonicBftTime error"),
        }
    }

    #[test]
    fn test_is_monotonic_height() {
        let val = vec![Validator::new("val-1")];
        let header_one = Header::new(&val).generate().unwrap();
        let header_two = Header::new(&val).height(2).generate().unwrap();

        let vp = ProdPredicates;

        // 1. ensure valid header verifies
        let result_ok = vp.is_monotonic_height(header_two.height, header_one.height);
        assert!(result_ok.is_ok());

        // 2. ensure header with non-monotonic height fails
        let result_err = vp.is_monotonic_height(header_one.height, header_two.height);

        match result_err {
            Err(VerificationError(VerificationErrorDetail::NonIncreasingHeight(e), _)) => {
                assert_eq!(e.got, header_one.height);
                assert_eq!(e.expected, header_two.height.increment());
            },
            _ => panic!("expected NonIncreasingHeight error"),
        }
    }

    #[test]
    fn test_is_matching_chain_id() {
        let val = vec![Validator::new("val-1")];
        let header_one = Header::new(&val).chain_id("chaina-1").generate().unwrap();
        let header_two = Header::new(&val).chain_id("chainb-1").generate().unwrap();

        let vp = ProdPredicates;

        // 1. ensure valid header verifies
        let result_ok = vp.is_matching_chain_id(&header_one.chain_id, &header_one.chain_id);
        assert!(result_ok.is_ok());

        // 2. ensure header with different chain-id fails
        let result_err = vp.is_matching_chain_id(&header_one.chain_id, &header_two.chain_id);

        match result_err {
            Err(VerificationError(VerificationErrorDetail::ChainIdMismatch(e), _)) => {
                assert_eq!(e.got, header_one.chain_id.to_string());
                assert_eq!(e.expected, header_two.chain_id.to_string());
            },
            _ => panic!("expected ChainIdMismatch error"),
        }
    }

    #[test]
    fn test_is_within_trust_period() {
        let val = Validator::new("val-1");
        let header = Header::new(&[val]).generate().unwrap();

        let vp = ProdPredicates;

        // 1. ensure valid header verifies
        let mut trusting_period = Duration::new(1000, 0);
        let now = OffsetDateTime::now_utc().try_into().unwrap();

        let result_ok = vp.is_within_trust_period(header.time, trusting_period, now);
        assert!(result_ok.is_ok());

        // 2. ensure header outside trusting period fails
        trusting_period = Duration::new(0, 1);

        let result_err = vp.is_within_trust_period(header.time, trusting_period, now);

        let expires_at = (header.time + trusting_period).unwrap();
        match result_err {
            Err(VerificationError(VerificationErrorDetail::NotWithinTrustPeriod(e), _)) => {
                assert_eq!(e.expires_at, expires_at);
                assert_eq!(e.now, now);
            },
            _ => panic!("expected NotWithinTrustPeriod error"),
        }
    }

    #[test]
    fn test_is_header_from_past() {
        let val = Validator::new("val-1");
        let header = Header::new(&[val]).generate().unwrap();

        let vp = ProdPredicates;
        let one_second = Duration::new(1, 0);

        let now = OffsetDateTime::now_utc().try_into().unwrap();

        // 1. ensure valid header verifies
        let result_ok = vp.is_header_from_past(header.time, one_second, now);

        assert!(result_ok.is_ok());

        // 2. ensure it fails if header is from a future time
        let now = (now - one_second * 15).unwrap();
        let result_err = vp.is_header_from_past(header.time, one_second, now);

        match result_err {
            Err(VerificationError(VerificationErrorDetail::HeaderFromTheFuture(e), _)) => {
                assert_eq!(e.header_time, header.time);
                assert_eq!(e.now, now);
            },
            _ => panic!("expected HeaderFromTheFuture error"),
        }
    }

    #[test]
    // NOTE: tests both current valset and next valset
    fn test_validator_sets_match() {
        let mut light_block: LightBlock =
            TestgenLightBlock::new_default(1).generate().unwrap().into();

        let bad_validator_set = ValidatorSet::new(vec!["bad-val"]).generate().unwrap();

        let vp = ProdPredicates;

        // Test positive case
        // 1. For predicate: validator_sets_match
        let val_sets_match_ok = vp.validator_sets_match(
            &light_block.validators,
            light_block.signed_header.header.validators_hash,
        );

        assert!(val_sets_match_ok.is_ok());

        // 2. For predicate: next_validator_sets_match
        let next_val_sets_match_ok = vp.next_validators_match(
            &light_block.next_validators,
            light_block.signed_header.header.next_validators_hash,
        );

        assert!(next_val_sets_match_ok.is_ok());

        // Test negative case
        // 1. For predicate: validator_sets_match
        light_block.validators = bad_validator_set.clone();

        let val_sets_match_err = vp.validator_sets_match(
            &light_block.validators,
            light_block.signed_header.header.validators_hash,
        );

        match val_sets_match_err {
            Err(VerificationError(VerificationErrorDetail::InvalidValidatorSet(e), _)) => {
                assert_eq!(
                    e.header_validators_hash,
                    light_block.signed_header.header.validators_hash
                );
                assert_eq!(e.validators_hash, light_block.validators.hash());
            },
            _ => panic!("expected InvalidValidatorSet error"),
        }

        // 2. For predicate: next_validator_sets_match
        light_block.next_validators = bad_validator_set;
        let next_val_sets_match_err = vp.next_validators_match(
            &light_block.next_validators,
            light_block.signed_header.header.next_validators_hash,
        );

        match next_val_sets_match_err {
            Err(VerificationError(VerificationErrorDetail::InvalidNextValidatorSet(e), _)) => {
                assert_eq!(
                    e.header_next_validators_hash,
                    light_block.signed_header.header.next_validators_hash
                );
                assert_eq!(e.next_validators_hash, light_block.next_validators.hash());
            },
            _ => panic!("expected InvalidNextValidatorSet error"),
        }
    }

    #[test]
    fn test_header_matches_commit() {
        let mut signed_header = TestgenLightBlock::new_default(1)
            .generate()
            .unwrap()
            .signed_header;

        let vp = ProdPredicates;

        // 1. ensure valid signed header verifies
        let result_ok =
            vp.header_matches_commit(&signed_header.header, signed_header.commit.block_id.hash);

        assert!(result_ok.is_ok());

        // 2. ensure invalid signed header fails
        signed_header.commit.block_id.hash =
            "15F15EF50BDE2018F4B129A827F90C18222C757770C8295EB8EE7BF50E761BC0"
                .parse()
                .unwrap();
        let result_err =
            vp.header_matches_commit(&signed_header.header, signed_header.commit.block_id.hash);

        // 3. ensure it fails with: VerificationVerificationError::InvalidCommitValue
        let header_hash = signed_header.header.hash();

        match result_err {
            Err(VerificationError(VerificationErrorDetail::InvalidCommitValue(e), _)) => {
                assert_eq!(e.header_hash, header_hash);
                assert_eq!(e.commit_hash, signed_header.commit.block_id.hash);
            },
            _ => panic!("expected InvalidCommitValue error"),
        }
    }

    #[test]
    fn test_valid_commit() {
        let light_block: LightBlock = TestgenLightBlock::new_default(1).generate().unwrap().into();

        let mut signed_header = light_block.signed_header;
        let val_set = light_block.validators;

        let vp = ProdPredicates;
        let commit_validator = ProdCommitValidator;

        // Test scenarios -->
        // 1. valid commit - must result "Ok"
        let mut result_ok = vp.valid_commit(&signed_header, &val_set, &commit_validator);

        assert!(result_ok.is_ok());

        // 2. no commit signatures - must return error
        let signatures = signed_header.commit.signatures.clone();
        signed_header.commit.signatures = vec![];

        let mut result_err = vp.valid_commit(&signed_header, &val_set, &commit_validator);

        match result_err {
            Err(VerificationError(VerificationErrorDetail::NoSignatureForCommit(_), _)) => {},
            _ => panic!("expected ImplementationSpecific error"),
        }

        // 3. commit.signatures.len() != validator_set.validators().len()
        // must return error
        let mut bad_sigs = vec![signatures.clone().swap_remove(1)];
        signed_header.commit.signatures.clone_from(&bad_sigs);

        result_err = vp.valid_commit(&signed_header, &val_set, &commit_validator);

        match result_err {
            Err(VerificationError(VerificationErrorDetail::MismatchPreCommitLength(e), _)) => {
                assert_eq!(e.pre_commit_length, signed_header.commit.signatures.len());
                assert_eq!(e.validator_length, val_set.validators().len());
            },
            _ => panic!("expected MismatchPreCommitLength error"),
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

                assert_eq!(e.validator_set, val_set_with_faulty_signer);
            },
            _ => panic!("expected FaultySigner error"),
        }
    }

    #[test]
    fn test_valid_next_validator_set() {
        let test_lb1 = TestgenLightBlock::new_default(1);
        let light_block1: LightBlock = test_lb1.generate().unwrap().into();

        let light_block2: LightBlock = test_lb1.next().generate().unwrap().into();

        let vp = ProdPredicates;

        // Test scenarios -->
        // 1. next_validator_set hash matches
        let result_ok = vp.valid_next_validator_set(
            light_block1.signed_header.header.validators_hash,
            light_block2.signed_header.header.next_validators_hash,
        );

        assert!(result_ok.is_ok());

        // 2. next_validator_set hash doesn't match
        let vals = &[Validator::new("new-1"), Validator::new("new-2")];
        let header = Header::new(vals);
        let commit = Commit::new(header.clone(), 1);

        let light_block3: LightBlock = TestgenLightBlock::new(header, commit)
            .generate()
            .unwrap()
            .into();

        let result_err = vp.valid_next_validator_set(
            light_block3.signed_header.header.validators_hash,
            light_block2.signed_header.header.next_validators_hash,
        );

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
            },
            _ => panic!("expected InvalidNextValidatorSet error"),
        }
    }

    #[test]
    fn test_has_sufficient_validators_overlap() {
        let light_block: LightBlock = TestgenLightBlock::new_default(1).generate().unwrap().into();
        let val_set = light_block.validators;
        let signed_header = light_block.signed_header;

        let vp = ProdPredicates;
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
            Err(VerificationError(VerificationErrorDetail::NotEnoughTrust(e), _)) => {
                assert_eq!(
                    e.tally,
                    VotingPowerTally {
                        total: 200,
                        tallied: 100,
                        trust_threshold,
                    }
                );
            },
            _ => panic!("expected NotEnoughTrust error"),
        }
    }

    #[test]
    fn test_has_sufficient_signers_overlap() {
        let mut light_block: LightBlock =
            TestgenLightBlock::new_default(2).generate().unwrap().into();

        let vp = ProdPredicates;
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
            Err(VerificationError(VerificationErrorDetail::InsufficientSignersOverlap(e), _)) => {
                assert_eq!(
                    e.tally,
                    VotingPowerTally {
                        total: 100,
                        tallied: 50,
                        trust_threshold,
                    }
                );
            },
            _ => panic!("expected InsufficientSignersOverlap error"),
        }
    }
}
