use light_spike::components::scheduler;
use light_spike::predicates::production::ProductionPredicates;
use light_spike::prelude::*;

use std::collections::HashMap;

pub fn main() {
    color_backtrace::install();

    let (trusted_store_reader, mut trusted_store_writer) = Store::new().split();
    let (untrusted_store_reader, untrusted_store_writer) = Store::new().split();

    let primary: Peer = "tcp://127.0.0.1:26657".parse().unwrap();
    let mut io = RealIo::new();

    let IoOutput::FetchedLightBlock(trusted_state) =
        io.fetch_light_block(primary.clone(), 1520).unwrap();

    trusted_store_writer.add(trusted_state);

    let peers = Peers {
        primary,
        witnesses: Vec::new(),
    };

    let state = State {
        peers,
        trusted_store_reader,
        trusted_store_writer,
        untrusted_store_reader,
        untrusted_store_writer,
        verification_trace: HashMap::new(),
    };

    let options = VerificationOptions {
        trust_threshold: TrustThreshold {
            numerator: 1,
            denominator: 3,
        },
        trusting_period: Duration::from_secs(36000),
        now: Time::now(),
    };

    let predicates = MockPredicates;
    let voting_power_calculator = MockVotingPower;
    let commit_validator = MockCommitValidator;
    let header_hasher = MockHeaderHasher;

    let verifier = RealVerifier::new(
        predicates,
        voting_power_calculator,
        commit_validator,
        header_hasher,
    );

    let clock = SystemClock;
    let scheduler = scheduler::schedule;
    let fork_detector = RealForkDetector::new(header_hasher);

    let mut demuxer = Demuxer::new(
        state,
        options,
        clock,
        scheduler,
        verifier,
        fork_detector,
        io,
    );

    demuxer.run().unwrap();
}

#[derive(Copy, Clone)]
struct MockHeaderHasher;
impl HeaderHasher for MockHeaderHasher {
    fn hash(&self, header: &Header) -> Hash {
        header.consensus_hash // FIXME: wrong hash
    }
}

#[derive(Copy, Clone)]
struct MockCommitValidator;
impl CommitValidator for MockCommitValidator {
    fn validate(
        &self,
        _commit: &Commit,
        _validators: &ValidatorSet,
    ) -> Result<(), anomaly::BoxError> {
        // let first_byte = commit.header_hash.as_bytes()[0];
        // if first_byte < 90 {
        //     return Err("invalid commit".into());
        // }

        Ok(())
    }
}

#[derive(Copy, Clone)]
struct MockVotingPower;
impl VotingPowerCalculator for MockVotingPower {
    fn total_power_of(&self, _validators: &ValidatorSet) -> u64 {
        8
    }
    fn voting_power_in(&self, _commit: &Commit, _validators: &ValidatorSet) -> u64 {
        4
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MockPredicates;

impl VerificationPredicates for MockPredicates {
    fn validator_sets_match(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<(), VerificationError> {
        ProductionPredicates.validator_sets_match(signed_header, validators)
    }

    fn next_validators_match(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<(), VerificationError> {
        ProductionPredicates.next_validators_match(signed_header, validators)
    }

    fn header_matches_commit(
        &self,
        header: &Header,
        commit: &Commit,
        header_hasher: &dyn HeaderHasher,
    ) -> Result<(), VerificationError> {
        ProductionPredicates.header_matches_commit(header, commit, header_hasher)
    }

    fn valid_commit(
        &self,
        commit: &Commit,
        validators: &ValidatorSet,
        validator: &dyn CommitValidator,
    ) -> Result<(), VerificationError> {
        ProductionPredicates.valid_commit(commit, validators, validator)
    }

    fn is_within_trust_period(
        &self,
        header: &Header,
        trusting_period: Duration,
        now: Time,
    ) -> Result<(), VerificationError> {
        ProductionPredicates.is_within_trust_period(header, trusting_period, now)
    }

    fn is_monotonic_bft_time(
        &self,
        untrusted_header: &Header,
        trusted_header: &Header,
    ) -> Result<(), VerificationError> {
        ProductionPredicates.is_monotonic_bft_time(untrusted_header, trusted_header)
    }

    fn is_monotonic_height(
        &self,
        untrusted_header: &Header,
        trusted_header: &Header,
    ) -> Result<(), VerificationError> {
        ProductionPredicates.is_monotonic_height(untrusted_header, trusted_header)
    }

    fn has_sufficient_voting_power(
        &self,
        commit: &Commit,
        validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError> {
        let first_byte = commit.header_hash.as_bytes()[0];

        if first_byte > 140 {
            return Err(VerificationError::InsufficientVotingPower {
                total_power: 0,
                voting_power: 0,
            });
        }

        ProductionPredicates.has_sufficient_voting_power(
            commit,
            validators,
            trust_threshold,
            calculator,
        )
    }

    fn has_sufficient_validators_overlap(
        &self,
        untrusted_commit: &Commit,
        trusted_validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError> {
        ProductionPredicates.has_sufficient_validators_overlap(
            untrusted_commit,
            trusted_validators,
            trust_threshold,
            calculator,
        )
    }

    fn has_sufficient_signers_overlap(
        &self,
        untrusted_commit: &Commit,
        untrusted_validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError> {
        ProductionPredicates.has_sufficient_signers_overlap(
            untrusted_commit,
            untrusted_validators,
            trust_threshold,
            calculator,
        )
    }

    fn valid_next_validator_set(
        &self,
        untrusted_sh: &SignedHeader,
        untrusted_next_vals: &ValidatorSet,
    ) -> Result<(), VerificationError> {
        ProductionPredicates.valid_next_validator_set(untrusted_sh, untrusted_next_vals)
    }
}
