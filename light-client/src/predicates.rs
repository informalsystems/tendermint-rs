//! Predicates for light block validation and verification.

use crate::{
    ensure,
    light_client::Options,
    operations::{CommitValidator, Hasher, VotingPowerCalculator},
    types::{Header, LightBlock, SignedHeader, Time, TrustThreshold, ValidatorSet},
};

use errors::VerificationError;
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
pub trait VerificationPredicates: Send {
    /// Compare the provided validator_set_hash against the hash produced from hashing the validator
    /// set.
    fn validator_sets_match(
        &self,
        light_block: &LightBlock,
        hasher: &dyn Hasher,
    ) -> Result<(), VerificationError> {
        let validators_hash = hasher.hash_validator_set(&light_block.validators);

        ensure!(
            light_block.signed_header.header.validators_hash == validators_hash,
            VerificationError::InvalidValidatorSet {
                header_validators_hash: light_block.signed_header.header.validators_hash,
                validators_hash,
            }
        );

        Ok(())
    }

    /// Check that the hash of the next validator set in the header match the actual one.
    fn next_validators_match(
        &self,
        light_block: &LightBlock,
        hasher: &dyn Hasher,
    ) -> Result<(), VerificationError> {
        let next_validators_hash = hasher.hash_validator_set(&light_block.next_validators);

        ensure!(
            light_block.signed_header.header.next_validators_hash == next_validators_hash,
            VerificationError::InvalidNextValidatorSet {
                header_next_validators_hash: light_block.signed_header.header.next_validators_hash,
                next_validators_hash,
            }
        );

        Ok(())
    }

    /// Check that the hash of the header in the commit matches the actual one.
    fn header_matches_commit(
        &self,
        signed_header: &SignedHeader,
        hasher: &dyn Hasher,
    ) -> Result<(), VerificationError> {
        let header_hash = hasher.hash_header(&signed_header.header);

        ensure!(
            header_hash == signed_header.commit.block_id.hash,
            VerificationError::InvalidCommitValue {
                header_hash,
                commit_hash: signed_header.commit.block_id.hash,
            }
        );

        Ok(())
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
        ensure!(
            expires_at > now,
            VerificationError::NotWithinTrustPeriod { expires_at, now }
        );

        Ok(())
    }

    /// Check that the untrusted header is from past.
    fn is_header_from_past(
        &self,
        untrusted_header: &Header,
        clock_drift: Duration,
        now: Time,
    ) -> Result<(), VerificationError> {
        ensure!(
            untrusted_header.time < now + clock_drift,
            VerificationError::HeaderFromTheFuture {
                header_time: untrusted_header.time,
                now
            }
        );

        Ok(())
    }

    /// Check that time passed monotonically between the trusted header and the untrusted one.
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

    /// Check that the height increased between the trusted header and the untrusted one.
    fn is_monotonic_height(
        &self,
        untrusted_header: &Header,
        trusted_header: &Header,
    ) -> Result<(), VerificationError> {
        let trusted_height = trusted_header.height;

        ensure!(
            untrusted_header.height > trusted_header.height,
            VerificationError::NonIncreasingHeight {
                got: untrusted_header.height,
                expected: trusted_height.increment(),
            }
        );

        Ok(())
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
        ensure!(
            light_block.signed_header.header.validators_hash
                == trusted_state.signed_header.header.next_validators_hash,
            VerificationError::InvalidNextValidatorSet {
                header_next_validators_hash: light_block.signed_header.header.validators_hash,
                next_validators_hash: trusted_state.signed_header.header.next_validators_hash,
            }
        );

        Ok(())
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
    use tendermint_testgen::{Validator, Header, Generator, Commit};
    use crate::predicates::{ProdPredicates, VerificationPredicates};
    use std::time::Duration;
    use tendermint::Time;
    use std::ops::Sub;
    use crate::tests::default_peer_id;
    use tendermint_testgen::validator::{generate_validator_set, generate_validators};
    use crate::operations::{ProdHasher, Hasher, ProdCommitValidator};
    use crate::predicates::errors::VerificationError;
    use crate::types::{PeerId, LightBlock, ValidatorSet, SignedHeader};
    use tendermint::block::{CommitSigs, CommitSig};

    #[test]
    fn test_is_monotonic_bft_time() {
        let test_val = vec![Validator::new("val-1")];
        let trusted_header = Header::new(&test_val)
            .generate();
        let untrusted_header = Header::new(&test_val)
            .generate();

        match (trusted_header, untrusted_header) {
            (Ok(trusted), Ok(untrusted)) => {
                let vp = ProdPredicates::default();
                let case_positive = vp.is_monotonic_bft_time(
                    &untrusted,
                    &trusted);
                assert!(case_positive.is_ok());

                let case_negative = vp.is_monotonic_bft_time(
                    &trusted,
                    &untrusted);
                assert!(case_negative.is_err());

                let error = VerificationError::NonMonotonicBftTime {
                    header_bft_time: trusted.time,
                    trusted_header_bft_time: untrusted.time,
                };
                assert_eq!(case_negative.err().unwrap(), error);

            }
            _ => println!("Error in generating header")
        }

    }

    #[test]
    fn test_is_within_trust_period() {
        let val = vec![Validator::new("val-1")];
        let header = Header::new(&val)
            .generate();

        match header {
            Ok(header) => {
                let vp = ProdPredicates::default();

                let mut trusting_period = Duration::new(1000,0);
                let case_positive = vp.is_within_trust_period(
                    &header.clone(),
                    trusting_period,
                    Time::now(),
                );
                assert!(case_positive.is_ok());

                trusting_period = Duration::new(0,1);
                let now = Time::now();
                let case_negative = vp.is_within_trust_period(
                    &header,
                    trusting_period,
                    now,
                );
                assert!(case_negative.is_err());

                let expires_at = header.time + trusting_period;
                let error = VerificationError::NotWithinTrustPeriod { expires_at, now };
                assert_eq!(case_negative.err().unwrap(), error);

            }
            Err(e) => println!("Error in generating header: {}", e)
        }

    }

    #[test]
    fn test_is_header_from_past() {
        let val = Validator::new("val-1");
        let header = Header::new([val.clone()].as_ref())
            .generate();

        match header {
            Ok(header) => {
                let vp = ProdPredicates::default();

                let one_second = Duration::new(1,0);
                let case_positive = vp.is_header_from_past(
                    &header.clone(),
                    one_second,
                    Time::now());

                assert!(case_positive.is_ok());

                let now = Time::now().sub(one_second * 5);
                let case_negative = vp.is_header_from_past(
                    &header,
                    one_second,
                    now,
                );

                assert!(case_negative.is_err());

                let error = VerificationError::HeaderFromTheFuture {
                    header_time: header.time,
                    now,
                };

                assert_eq!(case_negative.err().unwrap(), error);

            }
            Err(e) => println!("Error in generating header: {}", e)
        }
    }

    #[test]
    fn test_validator_sets_match() {
        let raw_vals = vec![Validator::new("val-1")];
        let light_block = generate_default_light_block(
            raw_vals,
            default_peer_id(),
        );

        let val_set_result = generate_validator_set(vec!["bad-val"]);

        match (light_block, val_set_result) {
            (
                Ok(mut light_block),
                Ok(bad_validator_set)
            )=> {

                let vp = ProdPredicates::default();
                let hasher = ProdHasher::default();

                // Test positive case
                // 1. For predicate validator_sets_match
                let val_sets_match_ok = vp.validator_sets_match(
                    &light_block,
                    &hasher
                );

                assert!(val_sets_match_ok.is_ok());

                // 2. For predicate next_validator_sets_match
                let next_val_sets_match_ok = vp.next_validators_match(
                    &light_block,
                    &hasher
                );

                assert!(next_val_sets_match_ok.is_ok());

                // Test negative case
                // 1. For predicate validator_sets_match
                light_block.validators = bad_validator_set.0.clone();

                let val_sets_match_err = vp.validator_sets_match(
                    &light_block,
                    &hasher
                );

                let val_set_error = VerificationError::InvalidValidatorSet {
                    header_validators_hash: light_block.signed_header.header.validators_hash,
                    validators_hash: hasher.hash_validator_set(&light_block.validators)
                };
                assert!(val_sets_match_err.is_err());
                assert_eq!(val_sets_match_err.err().unwrap(), val_set_error);

                // 2. For predicate next_validator_sets_match
                light_block.next_validators = bad_validator_set.0;
                let next_val_sets_match_err = vp.next_validators_match(
                    &light_block,
                    &hasher
                );

                let next_val_set_error = VerificationError::InvalidNextValidatorSet {
                    header_next_validators_hash: light_block.signed_header.header.next_validators_hash,
                    next_validators_hash: hasher.hash_validator_set(&light_block.next_validators)
                };

                assert!(next_val_sets_match_err.is_err());
                assert_eq!(next_val_sets_match_err.err().unwrap(), next_val_set_error);

            }
            (Err(e), _) => println!("Error in generating light block: {}", e),
            (_, Err(e)) => println!("Error in generating validator set: {}", e),
        }

    }

    #[test]
    fn test_header_matches_commit() {
        let raw_val = Validator::new("val-1");
        let raw_header = Header::new(&[raw_val]);
        let raw_commit = Commit::new(raw_header.clone(), 1);
        let signed_header = generate_signed_header(
            raw_header,
            raw_commit
        );

        match signed_header {
            Ok(mut signed_header) => {
                let vp = ProdPredicates::default();
                let hasher = ProdHasher::default();

                // We need to do this because the commit generator does not add BlockId to it currently
                // It keeps it "empty" because we don't have a way to make parts of the header
                // TODO: Should be removed once this is fixed in testgen!
                signed_header.commit.block_id.hash = hasher.hash_header(&signed_header.header);

                let result_ok = vp.header_matches_commit(
                    &signed_header,
                    &hasher
                );

                assert!(result_ok.is_ok());

                signed_header.commit.block_id.hash = "15F15EF50BDE2018F4B129A827F90C18222C757770C8295EB8EE7BF50E761BC0".parse().unwrap();
                let result_err = vp.header_matches_commit(
                    &signed_header,
                    &hasher
                );

                assert!(result_err.is_err());

                let header_hash = hasher.hash_header(&signed_header.header);
                let error = VerificationError::InvalidCommitValue {
                    header_hash,
                    commit_hash: signed_header.commit.block_id.hash,
                };

                assert_eq!(result_err.err().unwrap(), error);
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    #[test]
    fn test_valid_commit() {
        let (val_set, raw_val) = generate_validator_set(vec!["val-1","val-2"])
            .expect("Error generating validator set");
        let raw_header = Header::new(&raw_val);
        let raw_commit = Commit::new(raw_header.clone(), 1);
        let signed_header = generate_signed_header(
            raw_header,
            raw_commit
        );

        match signed_header {
            Ok(mut signed_header) => {
                let vp = ProdPredicates::default();
                let hasher = ProdHasher::default();
                let commit_validator = ProdCommitValidator::new(hasher);

                // Test various scenarios -->
                // 1. valid commit - must result "Ok"
                let mut result_ok = vp.valid_commit(
                    &signed_header,
                    &val_set,
                    &commit_validator
                );

                assert!(result_ok.is_ok());

                // 2. no commit signatures - must return error
                let signatures = signed_header.commit.signatures.clone();
                signed_header.commit.signatures = CommitSigs::default();
                let mut result_err = vp.valid_commit(
                    &signed_header,
                    &val_set,
                    &commit_validator
                );
                assert!(result_err.is_err());

                let error = VerificationError::ImplementationSpecific(
                        "no signatures for commit".to_string()
                );

                assert_eq!(result_err.err().unwrap(), error);

                // 3. commit.signatures.len() != validator_set.validators().len()
                // must return error
                let mut bad_sigs = vec![signatures.clone().into_vec().swap_remove(1)];
                signed_header.commit.signatures = CommitSigs::new::<Vec<CommitSig>>(
                    bad_sigs.clone()
                );

                result_err = vp.valid_commit(
                    &signed_header,
                    &val_set,
                    &commit_validator
                );
                assert!(result_err.is_err());

                let error = VerificationError::ImplementationSpecific(format!(
                    "pre-commit length: {} doesn't match validator length: {}",
                    signed_header.commit.signatures.len(),
                    val_set.validators().len()
                ));

                assert_eq!(result_err.err().unwrap(), error);

                // 4. commit.BlockIdFlagAbsent - should be "Ok"
                bad_sigs.push(CommitSig::BlockIDFlagAbsent);
                signed_header.commit.signatures = CommitSigs::new::<Vec<CommitSig>>(
                    bad_sigs
                );
                result_ok = vp.valid_commit(
                    &signed_header,
                    &val_set,
                    &commit_validator
                );
                assert!(result_ok.is_ok());

                // 5. faulty signer - must return error
                let val_set_with_faulty_signer = generate_validator_set(
                    vec!["val-1", "bad-val"]
                ).expect("Failed to generate validator set");

                result_err = vp.valid_commit(
                    &signed_header,
                    &val_set_with_faulty_signer.0,
                    &commit_validator
                );
                assert!(result_err.is_err());

            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    /// Helpers ->
    pub fn generate_default_light_block(
        raw_vals: Vec<Validator>,
        peer_id: PeerId,
    ) -> Result<LightBlock, String> {
        let raw_header = Header::new(&raw_vals);
        let raw_commit = Commit::new(raw_header.clone(), 1);

        let light_block = generate_light_block_with(
            raw_header,
            raw_commit,
            raw_vals,
            peer_id,
        );
        match light_block {
            Ok(light_block) => {
                Ok(light_block)
            }
            Err(e) => {
                Err(format!("{}",e))
            }
        }
    }

    pub fn generate_light_block_with(
        raw_header: Header,
        raw_commit: Commit,
        raw_vals: Vec<Validator>,
        peer_id: PeerId,
    ) -> Result<LightBlock, String> {
        let signed_header = generate_signed_header(raw_header, raw_commit);
        let vals = generate_validators(&raw_vals);

        match (signed_header, vals) {
            (Ok(signed_header), Ok(vals)) => {
                let validator_set = ValidatorSet::new(vals);

                let light_block = LightBlock::new(
                    signed_header,
                    validator_set.clone(), validator_set, peer_id);
                Ok(light_block)
            },
            (Err(e), _) => {
                Err(format!("Error: Failed to generate signed header: {} ", e))
            },
            (_, Err(e)) => {
                Err(format!("Error: Failed to generate validators: {} ", e))
            }
        }
    }

    pub fn generate_signed_header(
        raw_header: Header,
        raw_commit: Commit,
    ) -> Result<SignedHeader, String> {
        let header = raw_header.generate();
        let commit = raw_commit.generate();

        match (header, commit) {
            (
                Ok(header),
                Ok(commit)
            ) => {
                Ok(SignedHeader { header, commit })
            }
            _ => {
                Err(format!("Error: Failed to generate signed header!"))
            }
        }


    }
}