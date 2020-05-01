use light_spike::components::scheduler;
use light_spike::prelude::*;

pub fn main() {
    color_backtrace::install();

    let (trusted_store_reader, mut trusted_store_writer) = Store::new().split();
    let (untrusted_store_reader, untrusted_store_writer) = Store::new().split();

    let rpc_client = tendermint::rpc::Client::new("tcp://127.0.0.1:26657".parse().unwrap());
    let io = RealIo::new(rpc_client);

    let IoOutput::FetchedLightBlock(trusted_state) = io.fetch_light_block(1520).unwrap();
    trusted_store_writer.add(trusted_state);

    let state = State {
        trusted_store_reader,
        trusted_store_writer,
        untrusted_store_reader,
        untrusted_store_writer,
        errors: vec![],
    };

    let options = VerificationOptions {
        trust_threshold: TrustThreshold {
            numerator: 1,
            denominator: 3,
        },
        trusting_period: Duration::from_secs(36000),
        now: SystemTime::now(),
    };

    let predicates = light_spike::predicates::production::ProductionPredicates;
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

    demuxer.run();
}

#[derive(Copy, Clone)]
struct MockHeaderHasher;
impl HeaderHasher for MockHeaderHasher {
    fn hash(&self, header: &Header) -> Hash {
        header.hash
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
