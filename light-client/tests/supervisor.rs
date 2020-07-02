#![allow(unused_imports)]

use tendermint_light_client::{
    components::{
        clock::SystemClock,
        io::{AtHeight, Io, ProdIo},
        scheduler,
        verifier::ProdVerifier,
    },
    evidence::ProdEvidenceReporter,
    fork_detector::ProdForkDetector,
    light_client::{self, LightClient},
    peer_list::PeerList,
    state::State,
    store::{sled::SledStore, LightStore},
    supervisor::{Handle, Instance, Supervisor},
    types::{Height, LightBlock, PeerId, Status, Time, TrustThreshold},
};

use std::collections::HashMap;
use std::convert::TryInto;
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use futures::StreamExt;
use tendermint::abci::transaction::Hash;
use tendermint::evidence::Evidence;
use tendermint_light_client::components::io::IoError;
use tendermint_light_client::evidence::EvidenceReporter;
use tendermint_light_client::store::memory::MemoryStore;
use tendermint_light_client::tests::{
    random_peer_id, AnonLightBlock, MockClock, MockEvidenceReporter, MockIo, Provider,
    TestBisection, TrustOptions, WitnessProvider,
};

const TEST_FILES_PATH: &str = "./tests/support/";

fn read_json_fixture(file: impl AsRef<Path>) -> String {
    fs::read_to_string(file).unwrap()
}

fn load_multi_peer_testcases(dir: &str) -> Vec<TestBisection<LightBlock>> {
    let paths = fs::read_dir(PathBuf::from(TEST_FILES_PATH).join(dir)).unwrap();
    paths
        .into_iter()
        .flatten()
        .map(|entry| read_json_fixture(entry.path()).to_string())
        .map(|contents| serde_json::from_str::<TestBisection<AnonLightBlock>>(&contents).unwrap())
        .map(|testcase| testcase.into())
        .collect::<Vec<TestBisection<LightBlock>>>()
}

fn make_instance(
    peer_id: PeerId,
    trust_options: TrustOptions,
    provider: Provider<LightBlock>,
    now: Time,
) -> Instance {
    let io = MockIo::new(provider.chain_id, provider.lite_blocks);

    let trusted_height = trust_options.height.value();
    let trusted_state = io
        .fetch_light_block(peer_id, AtHeight::At(trusted_height))
        .expect("could not 'request' light block");

    let mut light_store = MemoryStore::new();
    light_store.insert(trusted_state, Status::Verified);

    let state = State {
        light_store: Box::new(light_store),
        verification_trace: HashMap::new(),
    };

    let options = light_client::Options {
        trust_threshold: trust_options.trust_level,
        trusting_period: trust_options.period.into(),
        clock_drift: Duration::from_secs(10),
        now,
    };

    let verifier = ProdVerifier::default();
    let clock = MockClock { now };
    let scheduler = scheduler::basic_bisecting_schedule;

    let light_client = LightClient::new(peer_id, options, clock, scheduler, verifier, io);

    Instance::new(light_client, state)
}

fn run_multipeer_test(tc: TestBisection<LightBlock>) {
    let primary = tc.primary.lite_blocks[0].provider;

    println!(
        "Running Test Case: {}\nwith Primary Peer: {:?}",
        tc.description, primary
    );

    let expects_err = match &tc.expected_output {
        Some(eo) => eo.eq("error"),
        None => false,
    };

    let primary_instance = make_instance(primary, tc.trust_options.clone(), tc.primary, tc.now);

    //- - - - - - - - - -

    let mut peer_list = PeerList::builder();
    peer_list = peer_list.primary(primary, primary_instance);

    for provider in tc.witnesses.iter() {
        let peer_id = provider.value.lite_blocks[0].provider;
        println!("Witness: {}", peer_id);
        let instance = make_instance(
            peer_id,
            tc.trust_options.clone(),
            provider.clone().value,
            tc.now,
        );
        peer_list = peer_list.witness(peer_id, instance);
    }

    //- - - - - - - - - -

    let mut supervisor = Supervisor::new(
        peer_list.build(),
        ProdForkDetector::default(),
        MockEvidenceReporter::new(),
    );

    // TODO: Add method to `Handle` to get a copy of the current peer list

    let mut handle = supervisor.handle();
    std::thread::spawn(|| supervisor.run());

    let target_height = tc.height_to_verify.try_into().unwrap();

    match handle.verify_to_target(target_height) {
        Ok(_new_states) => {
            assert!(!expects_err);
        }
        Err(e) => {
            dbg!(e);
            // if !expects_err {
            //     dbg!(e);
            // }

            assert!(expects_err);
        }
    }

    // TODO: Check the verdict
    // TODO: Check the peer list
    // TODO: Check we recorded a fork evidence (or not)
}

#[test]
fn deserialize_multi_peer_json() {
    load_multi_peer_testcases("bisection/multi_peer");
}

#[test]
fn run_multipeer_tests() {
    let testcases = load_multi_peer_testcases("bisection/multi_peer");
    for testcase in testcases {
        run_multipeer_test(testcase);
    }
}
