//! Provides an interface and default implementation for the `VotingPower` operation

use alloc::vec::Vec;
use core::{fmt, marker::PhantomData};

use serde::{Deserialize, Serialize};
use tendermint::{
    account,
    block::CommitSig,
    chain,
    crypto::signature,
    trust_threshold::TrustThreshold as _,
    validator,
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

impl VotingPowerTally {
    fn new(total: u64, trust_threshold: TrustThreshold) -> Self {
        Self {
            total,
            tallied: 0,
            trust_threshold,
        }
    }

    /// Adds given amount of power to tallied voting power amount.
    fn tally(&mut self, power: u64) {
        self.tallied += power;
        debug_assert!(self.tallied <= self.total);
    }

    /// Checks whether tallied amount meets trust threshold.
    fn check(&self) -> Result<(), Self> {
        if self
            .trust_threshold
            .is_enough_power(self.tallied, self.total)
        {
            Ok(())
        } else {
            Err(*self)
        }
    }
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

    /// Check that there is enough trust between an untrusted header and given
    /// trusted and untrusted validator sets.
    ///
    /// First of all, checks that enough validators from the
    /// `trusted_validators` set signed the `untrusted_header` to reach given
    /// `trust_threshold`.
    ///
    /// Second of all, checks that enough validators from the
    /// `untrusted_validators` set signed the `untrusted_header` to reach
    /// a trust threshold of ⅔.
    ///
    /// If both of those conditions aren’t met, it’s unspecified which error is
    /// returned.
    fn check_enough_trust_and_signers(
        &self,
        untrusted_header: &SignedHeader,
        trusted_validators: &ValidatorSet,
        trust_threshold: TrustThreshold,
        untrusted_validators: &ValidatorSet,
    ) -> Result<(), VerificationError> {
        let (trusted_power, untrusted_power) = self.voting_power_in_sets(
            untrusted_header,
            (trusted_validators, trust_threshold),
            (untrusted_validators, TrustThreshold::TWO_THIRDS),
        )?;
        trusted_power
            .check()
            .map_err(VerificationError::not_enough_trust)?;
        untrusted_power
            .check()
            .map_err(VerificationError::insufficient_signers_overlap)?;
        Ok(())
    }

    /// Check if there is 2/3rd overlap between an untrusted header and untrusted validator set
    fn check_signers_overlap(
        &self,
        untrusted_header: &SignedHeader,
        untrusted_validators: &ValidatorSet,
    ) -> Result<(), VerificationError> {
        let trust_threshold = TrustThreshold::TWO_THIRDS;
        self.voting_power_in(untrusted_header, untrusted_validators, trust_threshold)?
            .check()
            .map_err(VerificationError::insufficient_signers_overlap)
    }

    /// Compute the voting power in a header and its commit against a validator
    /// set.
    ///
    /// Note that the returned tally may be lower than actual tally so long as
    /// it meets the `trust_threshold`.  Furthermore, the method isn’t
    /// guaranteed to verify all the signatures present in the signed header.
    /// If there are invalid signatures, the method may or may not return an
    /// error depending on which validators those signatures correspond to.
    ///
    /// If you have two separate sets of validators and need to check voting
    /// power for both of them, prefer [`Self::voting_power_in_sets`] method.
    fn voting_power_in(
        &self,
        signed_header: &SignedHeader,
        validator_set: &ValidatorSet,
        trust_threshold: TrustThreshold,
    ) -> Result<VotingPowerTally, VerificationError>;

    /// Compute the voting power in a header and its commit against two separate
    /// validator sets.
    ///
    /// This is equivalent to calling [`Self::voting_power_in`] on each set
    /// separately but may be more optimised.  Implementators are encouraged to
    /// write a properly optimised method which avoids checking the same
    /// signature twice but for a simple unoptimised implementation the
    /// following works:
    ///
    /// ```ignore
    ///     fn voting_power_in_sets(
    ///         &self,
    ///         signed_header: &SignedHeader,
    ///         first_set: (&ValidatorSet, TrustThreshold),
    ///         second_set: (&ValidatorSet, TrustThreshold),
    ///     ) -> Result<(VotingPowerTally, VotingPowerTally), VerificationError> {
    ///         let first_tally = self.voting_power_in(
    ///             signed_header,
    ///             first_set.0,
    ///             first_set.1,
    ///         )?;
    ///         let second_tally = self.voting_power_in(
    ///             signed_header,
    ///             first_set.0,
    ///             first_set.1,
    ///         )?;
    ///         Ok((first_tally, second_tally))
    ///     }
    ///
    /// ```
    fn voting_power_in_sets(
        &self,
        signed_header: &SignedHeader,
        first_set: (&ValidatorSet, TrustThreshold),
        second_set: (&ValidatorSet, TrustThreshold),
    ) -> Result<(VotingPowerTally, VotingPowerTally), VerificationError>;
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

/// A signed non-nil vote.
struct NonAbsentCommitVote {
    signed_vote: SignedVote,
    /// Flag indicating whether the signature has already been verified.
    verified: bool,
}

impl NonAbsentCommitVote {
    /// Returns a signed non-nil vote for given commit.
    ///
    /// If the CommitSig represents a missing vote or a vote for nil returns
    /// `None`.  Otherwise, if the vote is missing a signature returns
    /// `Some(Err)`.  Otherwise, returns a `SignedVote` corresponding to given
    /// `CommitSig`.
    pub fn new(
        commit_sig: &CommitSig,
        validator_index: ValidatorIndex,
        commit: &Commit,
        chain_id: &chain::Id,
    ) -> Option<Result<Self, VerificationError>> {
        let (validator_address, timestamp, signature) = match commit_sig {
            CommitSig::BlockIdFlagAbsent { .. } => return None,
            CommitSig::BlockIdFlagCommit {
                validator_address,
                timestamp,
                signature,
            } => (*validator_address, *timestamp, signature),
            CommitSig::BlockIdFlagNil { .. } => return None,
        };

        let vote = Vote {
            vote_type: tendermint::vote::Type::Precommit,
            height: commit.height,
            round: commit.round,
            block_id: Some(commit.block_id),
            timestamp: Some(timestamp),
            validator_address,
            validator_index,
            signature: signature.clone(),
            extension: Default::default(),
            extension_signature: None,
        };
        Some(
            SignedVote::from_vote(vote, chain_id.clone())
                .ok_or_else(VerificationError::missing_signature)
                .map(|signed_vote| Self {
                    signed_vote,
                    verified: false,
                }),
        )
    }

    /// Returns address of the validator making the vote.
    pub fn validator_id(&self) -> account::Id {
        self.signed_vote.validator_id()
    }
}

/// Collection of non-absent commit votes.
struct NonAbsentCommitVotes {
    /// Votes sorted by validator address.
    votes: Vec<NonAbsentCommitVote>,
    /// Internal buffer for storing sign_bytes.
    ///
    /// The buffer is reused for each canonical vote so that we allocate it
    /// once.
    sign_bytes: Vec<u8>,
}

impl NonAbsentCommitVotes {
    /// Initial capacity of the `sign_bytes` buffer.
    ///
    /// The buffer will be resized if it happens to be too small so this value
    /// isn’t critical for correctness.  It’s a matter of performance to avoid
    /// reallocations.
    ///
    /// Note: As of protocol 0.38, maximum length of the sign bytes is `115 + (N > 13) + N`
    /// where `N` is the length of the chain id.
    /// Chain id can be at most 50 bytes (see [`tendermint::chain::id::MAX_LEN`])
    /// thus the largest buffer we’ll ever need is 166 bytes long.
    const SIGN_BYTES_INITIAL_CAPACITY: usize = 166;

    pub fn new(signed_header: &SignedHeader) -> Result<Self, VerificationError> {
        let mut votes = signed_header
            .commit
            .signatures
            .iter()
            .enumerate()
            .flat_map(|(idx, signature)| {
                // We never have more than 2³¹ signatures so this always
                // succeeds.
                let idx = ValidatorIndex::try_from(idx).unwrap();
                NonAbsentCommitVote::new(
                    signature,
                    idx,
                    &signed_header.commit,
                    &signed_header.header.chain_id,
                )
            })
            .collect::<Result<Vec<_>, VerificationError>>()?;
        votes.sort_unstable_by_key(NonAbsentCommitVote::validator_id);

        // Check if there are duplicate signatures.  If at least one duplicate
        // is found, report it as an error.
        let duplicate = votes
            .windows(2)
            .find(|pair| pair[0].validator_id() == pair[1].validator_id());
        if let Some(pair) = duplicate {
            Err(VerificationError::duplicate_validator(
                pair[0].validator_id(),
            ))
        } else {
            Ok(Self {
                votes,
                sign_bytes: Vec::with_capacity(Self::SIGN_BYTES_INITIAL_CAPACITY),
            })
        }
    }

    /// Looks up a vote cast by given validator.
    ///
    /// If the validator didn’t cast a vote or voted for `nil`, returns `Ok(None)`. Otherwise, if
    /// the vote had valid signature, returns `Ok(Some(idx))` where idx is the validator's index.
    /// If the vote had invalid signature, returns `Err`.
    pub fn has_voted<V: signature::Verifier>(
        &mut self,
        validator: &validator::Info,
    ) -> Result<Option<usize>, VerificationError> {
        if let Ok(idx) = self
            .votes
            .binary_search_by_key(&validator.address, NonAbsentCommitVote::validator_id)
        {
            let vote = &mut self.votes[idx];

            if !vote.verified {
                self.sign_bytes.clear();
                vote.signed_vote
                    .sign_bytes_into(&mut self.sign_bytes)
                    .expect("buffer is resized if needed and encoding never fails");

                let sign_bytes = self.sign_bytes.as_slice();
                validator
                    .verify_signature::<V>(sign_bytes, vote.signed_vote.signature())
                    .map_err(|_| {
                        VerificationError::invalid_signature(
                            vote.signed_vote.signature().as_bytes().to_vec(),
                            Box::new(validator.clone()),
                            sign_bytes.to_vec(),
                        )
                    })?;

                vote.verified = true;
            }

            Ok(Some(idx))
        } else {
            Ok(None)
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
        let mut votes = NonAbsentCommitVotes::new(signed_header)?;
        voting_power_in_impl::<V>(
            &mut votes,
            validator_set,
            trust_threshold,
            self.total_power_of(validator_set),
        )
    }

    fn voting_power_in_sets(
        &self,
        signed_header: &SignedHeader,
        first_set: (&ValidatorSet, TrustThreshold),
        second_set: (&ValidatorSet, TrustThreshold),
    ) -> Result<(VotingPowerTally, VotingPowerTally), VerificationError> {
        let mut votes = NonAbsentCommitVotes::new(signed_header)?;
        let first_tally = voting_power_in_impl::<V>(
            &mut votes,
            first_set.0,
            first_set.1,
            self.total_power_of(first_set.0),
        )?;
        let second_tally = voting_power_in_impl::<V>(
            &mut votes,
            second_set.0,
            second_set.1,
            self.total_power_of(second_set.0),
        )?;
        Ok((first_tally, second_tally))
    }
}

fn voting_power_in_impl<V: signature::Verifier>(
    votes: &mut NonAbsentCommitVotes,
    validator_set: &ValidatorSet,
    trust_threshold: TrustThreshold,
    total_voting_power: u64,
) -> Result<VotingPowerTally, VerificationError> {
    let mut power = VotingPowerTally::new(total_voting_power, trust_threshold);
    let mut seen_vals = Vec::new();

    for validator in validator_set.validators() {
        if let Some(idx) = votes.has_voted::<V>(validator)? {
            // Check if this validator has already voted.
            //
            // O(n) complexity.
            if seen_vals.contains(&idx) {
                return Err(VerificationError::duplicate_validator(validator.address));
            }
            seen_vals.push(idx);

            power.tally(validator.power());

            // Break early if sufficient voting power is reached.
            if power.check().is_ok() {
                break;
            }
        }
    }

    Ok(power)
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
