//! Provides an interface and default implementation for the `VotingPower` operation

use alloc::collections::BTreeSet as HashSet;
use core::{fmt, marker::PhantomData};

use serde::{Deserialize, Serialize};
use tendermint::{
    block::CommitSig,
    crypto::signature,
    trust_threshold::TrustThreshold as _,
    vote::{SignedVote, ValidatorIndex, Vote},
};

use crate::{
    errors::VerificationError,
    prelude::*,
    types::{Commit, SignedHeader, TrustThreshold, ValidatorSet},
};

/// Tally for the voting power computed by the `VotingPowerCalculator`
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct VotingPowerTally {
    /// Total voting power
    pub total: u64,
    /// Tallied voting power
    pub tallied: u64,
    /// Trust threshold for voting power
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

/// Computes the voting power in a commit against a validator set.
///
/// This trait provides default implementation of some helper functions.
pub trait VotingPowerCalculator: Send + Sync {
    /// Compute the total voting power in a validator set
    fn total_power_of(&self, validator_set: &ValidatorSet) -> u64 {
        validator_set
            .validators()
            .iter()
            .fold(0u64, |total, val_info| total + val_info.power.value())
    }

    /// Check against the given threshold that there is enough trust
    /// between an untrusted header and a trusted validator set
    fn check_enough_trust(
        &self,
        untrusted_header: &SignedHeader,
        trusted_validators: &ValidatorSet,
        trust_threshold: TrustThreshold,
    ) -> Result<(), VerificationError> {
        let voting_power =
            self.voting_power_in(untrusted_header, trusted_validators, trust_threshold)?;

        if trust_threshold.is_enough_power(voting_power.tallied, voting_power.total) {
            Ok(())
        } else {
            Err(VerificationError::not_enough_trust(voting_power))
        }
    }

    /// Check if there is 2/3rd overlap between an untrusted header and untrusted validator set
    fn check_signers_overlap(
        &self,
        untrusted_header: &SignedHeader,
        untrusted_validators: &ValidatorSet,
    ) -> Result<(), VerificationError> {
        let trust_threshold = TrustThreshold::TWO_THIRDS;
        let voting_power =
            self.voting_power_in(untrusted_header, untrusted_validators, trust_threshold)?;

        if trust_threshold.is_enough_power(voting_power.tallied, voting_power.total) {
            Ok(())
        } else {
            Err(VerificationError::insufficient_signers_overlap(
                voting_power,
            ))
        }
    }

    /// Compute the voting power in a header and its commit against a validator set.
    ///
    /// The `trust_threshold` is currently not used, but might be in the future
    /// for optimization purposes.
    fn voting_power_in(
        &self,
        signed_header: &SignedHeader,
        validator_set: &ValidatorSet,
        trust_threshold: TrustThreshold,
    ) -> Result<VotingPowerTally, VerificationError>;
}

/// Default implementation of a `VotingPowerCalculator`, parameterized with
/// the signature verification trait.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ProvidedVotingPowerCalculator<V> {
    _verifier: PhantomData<V>,
}

// Safety: the only member is phantom data
unsafe impl<V> Send for ProvidedVotingPowerCalculator<V> {}
unsafe impl<V> Sync for ProvidedVotingPowerCalculator<V> {}

impl<V> Default for ProvidedVotingPowerCalculator<V> {
    fn default() -> Self {
        Self {
            _verifier: PhantomData,
        }
    }
}

/// Default implementation of a `VotingPowerCalculator`.
#[cfg(feature = "rust-crypto")]
pub type ProdVotingPowerCalculator =
    ProvidedVotingPowerCalculator<tendermint::crypto::default::signature::Verifier>;

impl<V: signature::Verifier> VotingPowerCalculator for ProvidedVotingPowerCalculator<V> {
    fn voting_power_in(
        &self,
        signed_header: &SignedHeader,
        validator_set: &ValidatorSet,
        trust_threshold: TrustThreshold,
    ) -> Result<VotingPowerTally, VerificationError> {
        let signatures = &signed_header.commit.signatures;

        let mut tallied_voting_power = 0_u64;
        let mut seen_validators = HashSet::new();

        // Get non-absent votes from the signatures
        let non_absent_votes = signatures.iter().enumerate().flat_map(|(idx, signature)| {
            non_absent_vote(
                signature,
                ValidatorIndex::try_from(idx).unwrap(),
                &signed_header.commit,
            )
            .map(|vote| (signature, vote))
        });

        for (signature, vote) in non_absent_votes {
            // Ensure we only count a validator's power once
            if seen_validators.contains(&vote.validator_address) {
                return Err(VerificationError::duplicate_validator(
                    vote.validator_address,
                ));
            } else {
                seen_validators.insert(vote.validator_address);
            }

            let validator = match validator_set.validator(vote.validator_address) {
                Some(validator) => validator,
                None => continue, // Cannot find matching validator, so we skip the vote
            };

            let signed_vote =
                SignedVote::from_vote(vote.clone(), signed_header.header.chain_id.clone())
                    .ok_or_else(VerificationError::missing_signature)?;

            // Check vote is valid
            let sign_bytes = signed_vote.sign_bytes();
            if validator
                .verify_signature::<V>(&sign_bytes, signed_vote.signature())
                .is_err()
            {
                return Err(VerificationError::invalid_signature(
                    signed_vote.signature().as_bytes().to_vec(),
                    Box::new(validator),
                    sign_bytes,
                ));
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
    validator_index: ValidatorIndex,
    commit: &Commit,
) -> Option<Vote> {
    let (validator_address, timestamp, signature, block_id) = match commit_sig {
        CommitSig::BlockIdFlagAbsent { .. } => return None,
        CommitSig::BlockIdFlagCommit {
            validator_address,
            timestamp,
            signature,
        } => (
            *validator_address,
            *timestamp,
            signature,
            Some(commit.block_id),
        ),
        CommitSig::BlockIdFlagNil {
            validator_address,
            timestamp,
            signature,
        } => (*validator_address, *timestamp, signature, None),
    };

    Some(Vote {
        vote_type: tendermint::vote::Type::Precommit,
        height: commit.height,
        round: commit.round,
        block_id,
        timestamp: Some(timestamp),
        validator_address,
        validator_index,
        signature: signature.clone(),
        extension: Default::default(),
        extension_signature: None,
    })
}

// The below unit tests replaces the static voting power test files
// see https://github.com/informalsystems/tendermint-rs/pull/383
// This is essentially to remove the heavy dependency on MBT
// TODO: We plan to add Lightweight MBT for `voting_power_in` in the near future
#[cfg(test)]
mod tests {
    use tendermint::trust_threshold::TrustThresholdFraction;
    use tendermint_testgen::{
        light_block::generate_signed_header, Commit, Generator, Header,
        LightBlock as TestgenLightBlock, ValidatorSet, Vote as TestgenVote,
    };

    use super::*;
    use crate::{errors::VerificationErrorDetail, types::LightBlock};

    const EXPECTED_RESULT: VotingPowerTally = VotingPowerTally {
        total: 100,
        tallied: 0,
        trust_threshold: TrustThresholdFraction::ONE_THIRD,
    };

    #[test]
    fn test_empty_signatures() {
        let vp_calculator = ProdVotingPowerCalculator::default();
        let trust_threshold = TrustThreshold::default();

        let mut light_block: LightBlock = TestgenLightBlock::new_default(10)
            .generate()
            .unwrap()
            .into();
        light_block.signed_header.commit.signatures = vec![];

        let result_ok = vp_calculator.voting_power_in(
            &light_block.signed_header,
            &light_block.validators,
            trust_threshold,
        );

        // ensure the result matches the expected result
        assert_eq!(result_ok.unwrap(), EXPECTED_RESULT);
    }

    #[test]
    fn test_all_signatures_absent() {
        let vp_calculator = ProdVotingPowerCalculator::default();
        let trust_threshold = TrustThreshold::default();

        let mut testgen_lb = TestgenLightBlock::new_default(10);
        let mut commit = testgen_lb.commit.clone().unwrap();
        // an empty vector of votes translates into all absent signatures
        commit.votes = Some(vec![]);
        testgen_lb.commit = Some(commit);
        let light_block: LightBlock = testgen_lb.generate().unwrap().into();

        let result_ok = vp_calculator.voting_power_in(
            &light_block.signed_header,
            &light_block.validators,
            trust_threshold,
        );

        // ensure the result matches the expected result
        assert_eq!(result_ok.unwrap(), EXPECTED_RESULT);
    }

    #[test]
    fn test_all_signatures_nil() {
        let vp_calculator = ProdVotingPowerCalculator::default();
        let trust_threshold = TrustThreshold::default();

        let validator_set = ValidatorSet::new(vec!["a", "b"]);
        let vals = validator_set.clone().validators.unwrap();
        let header = Header::new(&vals);
        let votes = vec![
            TestgenVote::new(vals[0].clone(), header.clone()).nil(true),
            TestgenVote::new(vals[1].clone(), header.clone()).nil(true),
        ];
        let commit = Commit::new_with_votes(header.clone(), 1, votes);
        let signed_header = generate_signed_header(&header, &commit).unwrap();
        let valset = validator_set.generate().unwrap();

        let result_ok = vp_calculator.voting_power_in(&signed_header, &valset, trust_threshold);

        // ensure the result matches the expected result
        assert_eq!(result_ok.unwrap(), EXPECTED_RESULT);
    }

    #[test]
    fn test_one_invalid_signature() {
        let vp_calculator = ProdVotingPowerCalculator::default();
        let trust_threshold = TrustThreshold::default();

        let mut testgen_lb = TestgenLightBlock::new_default(10);
        let mut commit = testgen_lb.commit.clone().unwrap();
        let mut votes = commit.votes.unwrap();
        let vote = votes.pop().unwrap();
        let header = vote.clone().header.unwrap().chain_id("bad-chain");
        votes.push(vote.header(header));

        commit.votes = Some(votes);
        testgen_lb.commit = Some(commit);
        let light_block: LightBlock = testgen_lb.generate().unwrap().into();

        let result_err = vp_calculator.voting_power_in(
            &light_block.signed_header,
            &light_block.validators,
            trust_threshold,
        );

        match result_err {
            Err(VerificationError(VerificationErrorDetail::InvalidSignature(_), _)) => {},
            _ => panic!("expected InvalidSignature error"),
        }
    }

    #[test]
    fn test_all_signatures_invalid() {
        let vp_calculator = ProdVotingPowerCalculator::default();
        let trust_threshold = TrustThreshold::default();

        let mut testgen_lb = TestgenLightBlock::new_default(10);
        let header = testgen_lb.header.unwrap().chain_id("bad-chain");
        testgen_lb.header = Some(header);
        let light_block: LightBlock = testgen_lb.generate().unwrap().into();

        let result_err = vp_calculator.voting_power_in(
            &light_block.signed_header,
            &light_block.validators,
            trust_threshold,
        );

        match result_err {
            Err(VerificationError(VerificationErrorDetail::InvalidSignature(_), _)) => {},
            _ => panic!("expected InvalidSignature error"),
        }
    }

    #[test]
    fn test_signatures_from_diff_valset() {
        let vp_calculator = ProdVotingPowerCalculator::default();
        let trust_threshold = TrustThreshold::default();

        let mut light_block: LightBlock = TestgenLightBlock::new_default(10)
            .generate()
            .unwrap()
            .into();
        light_block.validators = ValidatorSet::new(vec!["bad-val1", "bad-val2"])
            .generate()
            .unwrap();

        let result_ok = vp_calculator.voting_power_in(
            &light_block.signed_header,
            &light_block.validators,
            trust_threshold,
        );

        // ensure the result matches the expected result
        assert_eq!(result_ok.unwrap(), EXPECTED_RESULT);
    }
}
