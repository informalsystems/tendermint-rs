use crate::{
    bail,
    predicates::errors::VerificationError,
    types::{LightBlock, TMCommit, TMLightBlock, TMSignedHeader, TMValidatorSet, TrustThreshold},
};

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

use tendermint::block::CommitSig;
use tendermint::lite::types::TrustThreshold as _;
use tendermint::vote::{SignedVote, Vote};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VotingPowerTally {
    pub total: u64,
    pub tallied: u64,
    pub trust_threshold: TrustThreshold,
}

impl fmt::Display for VotingPowerTally {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VotingPower(total={} tallied={} trust_threshold={})",
            self.total, self.tallied, self.trust_threshold
        )
    }
}

pub trait VotingPowerCalculator<LB: LightBlock>: Send {
    fn total_power_of(&self, validator_set: &LB::ValidatorSet) -> u64;

    fn voting_power_in(
        &self,
        signed_header: &LB::SignedHeader,
        validator_set: &LB::ValidatorSet,
        trust_threshold: TrustThreshold,
    ) -> Result<VotingPowerTally, VerificationError>;

    fn check_enough_trust(
        &self,
        untrusted_header: &LB::SignedHeader,
        trusted_validators: &LB::ValidatorSet,
        trust_threshold: TrustThreshold,
    ) -> Result<(), VerificationError> {
        let voting_power =
            self.voting_power_in(untrusted_header, trusted_validators, trust_threshold)?;

        if trust_threshold.is_enough_power(voting_power.tallied, voting_power.total) {
            Ok(())
        } else {
            Err(VerificationError::NotEnoughTrust(voting_power))
        }
    }

    fn check_signers_overlap(
        &self,
        untrusted_header: &LB::SignedHeader,
        untrusted_validators: &LB::ValidatorSet,
    ) -> Result<(), VerificationError> {
        let trust_threshold = TrustThreshold::TWO_THIRDS;
        let voting_power =
            self.voting_power_in(untrusted_header, untrusted_validators, trust_threshold)?;

        if trust_threshold.is_enough_power(voting_power.tallied, voting_power.total) {
            Ok(())
        } else {
            Err(VerificationError::InsufficientSignersOverlap(voting_power))
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ProdVotingPowerCalculator;

impl VotingPowerCalculator<TMLightBlock> for ProdVotingPowerCalculator {
    fn total_power_of(&self, validator_set: &TMValidatorSet) -> u64 {
        validator_set
            .validators()
            .iter()
            .fold(0u64, |total, val_info| {
                total + val_info.voting_power.value()
            })
    }

    fn voting_power_in(
        &self,
        signed_header: &TMSignedHeader,
        validator_set: &TMValidatorSet,
        trust_threshold: TrustThreshold,
    ) -> Result<VotingPowerTally, VerificationError> {
        let signatures = &signed_header.commit.signatures;

        let mut tallied_voting_power = 0_u64;
        let mut seen_validators = HashSet::new();

        // Get non-absent votes from the signatures
        let non_absent_votes = signatures.iter().enumerate().flat_map(|(idx, signature)| {
            if let Some(vote) = non_absent_vote(signature, idx as u64, &signed_header.commit) {
                Some((signature, vote))
            } else {
                None
            }
        });

        for (signature, vote) in non_absent_votes {
            // Ensure we only count a validator's power once
            if seen_validators.contains(&vote.validator_address) {
                bail!(VerificationError::DuplicateValidator(
                    vote.validator_address
                ));
            } else {
                seen_validators.insert(vote.validator_address);
            }

            let validator = match validator_set.validator(vote.validator_address) {
                Some(validator) => validator,
                None => continue, // Cannot find matching validator, so we skip the vote
            };

            let signed_vote = SignedVote::new(
                (&vote).into(),
                signed_header.header.chain_id.as_str(),
                vote.validator_address,
                vote.signature,
            );

            // Check vote is valid
            let sign_bytes = signed_vote.sign_bytes();
            if !validator.verify_signature(&sign_bytes, signed_vote.signature()) {
                bail!(VerificationError::InvalidSignature {
                    signature: signed_vote.signature().to_vec(),
                    validator,
                    sign_bytes,
                });
            }

            // If the vote is neither absent nor nil, tally its power
            if signature.is_commit() {
                tallied_voting_power += validator.power();
            } else {
                // It's OK. We include stray signatures (~votes for nil)
                // to measure validator availability.
            }

            // TODO: Break out of the loop when we have enough voting power.
            // See https://github.com/informalsystems/tendermint-rs/issues/235
        }

        let voting_power = VotingPowerTally {
            total: self.total_power_of(validator_set),
            tallied: tallied_voting_power,
            trust_threshold,
        };

        Ok(voting_power)
    }
}

fn non_absent_vote(
    commit_sig: &CommitSig,
    validator_index: u64,
    commit: &TMCommit,
) -> Option<Vote> {
    let (validator_address, timestamp, signature, block_id) = match commit_sig {
        CommitSig::BlockIDFlagAbsent { .. } => return None,
        CommitSig::BlockIDFlagCommit {
            validator_address,
            timestamp,
            signature,
        } => (
            *validator_address,
            *timestamp,
            signature.clone(),
            Some(commit.block_id.clone()),
        ),
        CommitSig::BlockIDFlagNil {
            validator_address,
            timestamp,
            signature,
        } => (*validator_address, *timestamp, signature.clone(), None),
    };

    Some(Vote {
        vote_type: tendermint::vote::Type::Precommit,
        height: commit.height,
        round: commit.round,
        block_id,
        timestamp,
        validator_address,
        validator_index,
        signature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::fs;
    use std::path::Path;

    const TEST_FILES_PATH: &str = "./tests/support/voting_power/";

    #[test]
    fn json_testcases() {
        run_all_tests();
    }

    #[derive(Debug, Deserialize)]
    enum TestResult {
        Ok { total: u64, tallied: u64 },
        Err { error_type: String },
    }

    #[derive(Debug, Deserialize)]
    struct TestCase {
        description: String,
        result: TestResult,
        signed_header: TMSignedHeader,
        validator_set: TMValidatorSet,
    }

    fn read_json_fixture(file: impl AsRef<Path>) -> String {
        fs::read_to_string(file).unwrap()
    }

    fn read_test_case(file_path: impl AsRef<Path>) -> TestCase {
        serde_json::from_str(read_json_fixture(file_path).as_str()).unwrap()
    }

    fn run_all_tests() {
        for entry in fs::read_dir(TEST_FILES_PATH).unwrap() {
            let entry = entry.unwrap();
            let tc = read_test_case(entry.path());
            let name = entry.file_name().to_string_lossy().to_string();
            run_test(tc, name);
        }
    }

    fn run_test(tc: TestCase, file: String) {
        println!("- Test '{}' in {}", tc.description, file);

        let calculator = ProdVotingPowerCalculator::default();
        let trust_threshold = TrustThreshold::default();

        let tally =
            calculator.voting_power_in(&tc.signed_header, &tc.validator_set, trust_threshold);

        match tc.result {
            TestResult::Ok { total, tallied } => {
                assert!(tally.is_ok(), "unexpected error");
                let tally = tally.unwrap();
                assert_eq!(tally.total, total);
                assert_eq!(tally.tallied, tallied);
            }
            TestResult::Err { error_type } => {
                assert!(tally.is_err());
                let err = tally.err().unwrap();
                let err_str = format!("{:?}", err);
                assert!(err_str.contains(&error_type));
            }
        }

        println!("  => SUCCESS");
    }
}
