use tendermint_light_client::{
    components::{
        io::{AtHeight, Io},
        scheduler,
        verifier::ProdVerifier,
    },
    fork_detector::ProdForkDetector,
    light_client::{self, LightClient},
    operations::ProdHasher,
    peer_list::PeerList,
    state::State,
    store::LightStore,
    supervisor::{Handle, Instance, Supervisor},
    types::{LightBlock, PeerId, Status, Time},
};

use std::collections::HashMap;
use std::time::Duration;

use tendermint_light_client::store::memory::MemoryStore;
use tendermint_light_client::tests::{
    LightClientTest, MockClock, MockEvidenceReporter, MockIo, TrustOptions,
};

use tendermint_testgen::Tester;

const TEST_FILES_PATH: &str = "./tests/support/";

fn make_instance(peer_id: PeerId, trust_options: TrustOptions, io: MockIo, now: Time) -> Instance {
    let trusted_height = trust_options.height;
    let trusted_state = io
        .fetch_light_block(AtHeight::At(trusted_height))
        .expect("could not 'request' light block");

    let mut light_store = MemoryStore::new();
    light_store.insert(trusted_state, Status::Trusted);

    let state = State {
        light_store: Box::new(light_store),
        verification_trace: HashMap::new(),
    };

    let options = light_client::Options {
        trust_threshold: trust_options.trust_level,
        trusting_period: trust_options.period.into(),
        clock_drift: Duration::from_secs(10),
    };

    let clock = MockClock { now };
    let verifier = ProdVerifier::default();
    let hasher = ProdHasher::default();
    let scheduler = scheduler::basic_bisecting_schedule;

    let light_client = LightClient::new(peer_id, options, clock, scheduler, verifier, hasher, io);

    Instance::new(light_client, state)
}

fn run_multipeer_test(tc: LightClientTest<LightBlock>) {
    let primary = tc.primary.lite_blocks[0].provider;

    println!(
        "Running Test Case: {}\nwith Primary Peer: {:?}",
        tc.description, primary
    );

    let expects_err = match &tc.expected_output {
        Some(eo) => eo.eq("error"),
        None => false,
    };

    let io = MockIo::new(tc.primary.chain_id, tc.primary.lite_blocks);
    let primary_instance = make_instance(primary, tc.trust_options.clone(), io.clone(), tc.now);

    let mut peer_list = PeerList::builder();
    peer_list.primary(primary, primary_instance);

    for provider in tc.witnesses.into_iter() {
        let peer_id = provider.value.lite_blocks[0].provider;
        println!("Witness: {}", peer_id);
        let io = MockIo::new(provider.value.chain_id, provider.value.lite_blocks);
        let instance = make_instance(peer_id, tc.trust_options.clone(), io.clone(), tc.now);
        peer_list.witness(peer_id, instance);
    }

    let supervisor = Supervisor::new(
        peer_list.build(),
        ProdForkDetector::default(),
        MockEvidenceReporter::new(),
    );

    // TODO: Add method to `Handle` to get a copy of the current peer list

    let handle = supervisor.handle();
    std::thread::spawn(|| supervisor.run());

    let target_height = tc.height_to_verify;

    match handle.verify_to_target(target_height) {
        Ok(new_state) => {
            // Check that the expected state and new_state match
            let untrusted_light_block = io
                .fetch_light_block(AtHeight::At(target_height))
                .expect("header at untrusted height not found");

            let expected_state = untrusted_light_block;
            assert_eq!(new_state.height(), expected_state.height());
            assert_eq!(new_state, expected_state);

            // Check the verdict
            assert!(!expects_err);
        }
        Err(e) => {
            dbg!(e);
            assert!(expects_err);
        }
    }

    // TODO: Check the peer list
    // TODO: Check we recorded a fork evidence (or not)
}

#[test]
fn run_multipeer_tests() {
    let mut tester = Tester::new("bisection_multi_peer", TEST_FILES_PATH);
    tester.add_test("multipeer test", run_multipeer_test);
    tester.run_foreach_in_dir("bisection/multi_peer");
    tester.finalize();
}
