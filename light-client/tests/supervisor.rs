use tendermint_light_client::{
    components::{
        io::{AtHeight, Io},
        scheduler,
        verifier::ProdVerifier,
    },
    fork_detector::ProdForkDetector,
    light_client::{self, LightClient},
    peer_list::PeerList,
    state::State,
    store::LightStore,
    supervisor::{Handle, Instance, Supervisor},
    types::{LightBlock, PeerId, Status, Time},
};

use std::collections::HashMap;
use std::convert::TryInto;
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use tendermint_light_client::store::memory::MemoryStore;
use tendermint_light_client::tests::{
    AnonLightBlock, MockClock, MockEvidenceReporter, MockIo, TestBisection, TrustOptions,
};

const TEST_FILES_PATH: &str = "./tests/support/";

fn read_json_fixture(file: impl AsRef<Path>) -> String {
    fs::read_to_string(file).unwrap()
}

fn load_multi_peer_testcases(dir: &str) -> Vec<TestBisection<LightBlock>> {
    let paths = fs::read_dir(PathBuf::from(TEST_FILES_PATH).join(dir)).unwrap();

    paths
        .flatten()
        .map(|entry| read_json_fixture(entry.path()))
        .map(|contents| serde_json::from_str::<TestBisection<AnonLightBlock>>(&contents).unwrap())
        .map(|testcase| testcase.into())
        .collect::<Vec<TestBisection<LightBlock>>>()
}

async fn make_instance(
    peer_id: PeerId,
    trust_options: TrustOptions,
    io: MockIo,
    now: Time,
) -> Instance {
    let trusted_height = trust_options.height.value();
    let trusted_state = io
        .fetch_light_block(peer_id, AtHeight::At(trusted_height))
        .await
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

    let verifier = ProdVerifier::default();
    let clock = MockClock { now };
    let scheduler = scheduler::basic_bisecting_schedule;

    let light_client = LightClient::new(peer_id, options, clock, scheduler, verifier, io);

    Instance::new(light_client, state)
}

async fn run_multipeer_test(tc: TestBisection<LightBlock>) {
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
    let primary_instance =
        make_instance(primary, tc.trust_options.clone(), io.clone(), tc.now).await;

    let mut peer_list = PeerList::builder();
    peer_list = peer_list.primary(primary, primary_instance);

    for provider in tc.witnesses.into_iter() {
        let peer_id = provider.value.lite_blocks[0].provider;
        println!("Witness: {}", peer_id);
        let io = MockIo::new(provider.value.chain_id, provider.value.lite_blocks);
        let instance = make_instance(peer_id, tc.trust_options.clone(), io.clone(), tc.now).await;
        peer_list = peer_list.witness(peer_id, instance);
    }

    let mut supervisor = Supervisor::new(
        peer_list.build(),
        ProdForkDetector::default(),
        MockEvidenceReporter::new(),
    );

    // TODO: Add method to `Handle` to get a copy of the current peer list

    let handle = supervisor.handle();
    std::thread::spawn(|| block_on(supervisor.run()));

    let target_height = tc.height_to_verify.try_into().unwrap();

    match handle.verify_to_target(target_height) {
        Ok(new_state) => {
            // Check that the expected state and new_state match
            let untrusted_light_block = io
                .fetch_light_block(primary, AtHeight::At(target_height))
                .await
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
fn deserialize_multi_peer_json() {
    load_multi_peer_testcases("bisection/multi_peer");
}

#[test]
fn run_multipeer_tests() {
    let testcases = load_multi_peer_testcases("bisection/multi_peer");
    for testcase in testcases {
        block_on(run_multipeer_test(testcase));
    }
}

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    let mut rt = tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(f)
}
