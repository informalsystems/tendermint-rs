use light_spike::components::scheduler;
use light_spike::predicates::production::ProdPredicates;
use light_spike::prelude::*;

use std::collections::HashMap;

pub fn main() {
    color_backtrace::install();

    let primary_addr: tendermint::net::Address = "tcp://127.0.0.1:26657".parse().unwrap();
    let primary: PeerId = "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE".parse().unwrap();
    let mut peer_map = HashMap::new();
    peer_map.insert(primary, primary_addr);

    let mut io = RealIo::new(peer_map);
    let trusted_state = io.fetch_light_block(primary.clone(), 9977).unwrap();

    let mut light_store = MemoryStore::new();
    light_store.insert(trusted_state, VerifiedStatus::Verified);

    let peers = Peers {
        primary,
        witnesses: Vec::new(),
    };

    let state = State {
        peers,
        light_store: Box::new(light_store),
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

    let verifier = ProdVerifier::new(
        predicates,
        voting_power_calculator,
        commit_validator,
        header_hasher,
    );

    let clock = SystemClock;
    let scheduler = scheduler::schedule;
    let fork_detector = RealForkDetector::new(header_hasher);

    let _demuxer = Demuxer::new(
        state,
        options,
        clock,
        scheduler,
        verifier,
        fork_detector,
        io,
    );

    todo!()
}

