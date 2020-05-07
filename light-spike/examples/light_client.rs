use light_spike::components::scheduler;
use light_spike::predicates::production::ProdPredicates;
use light_spike::prelude::*;

use std::collections::HashMap;

pub fn main() {
    color_backtrace::install();

    let (trusted_store_reader, mut trusted_store_writer) = Store::new().split();
    let (untrusted_store_reader, untrusted_store_writer) = Store::new().split();

    let primary: Peer = "tcp://127.0.0.1:26657".parse().unwrap();
    let mut io = RealIo::new();

    let trusted_state = io.fetch_light_block(primary.clone(), 9977).unwrap();

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

    let predicates = ProdPredicates;
    let voting_power_calculator = ProdVotingPowerCalculator;
    let commit_validator = ProdCommitValidator;
    let header_hasher = ProdHeaderHasher;

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

