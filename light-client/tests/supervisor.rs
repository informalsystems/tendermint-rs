#![allow(dead_code, unused_imports)]

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
    store::{sled::SledStore, LightStore, VerifiedStatus},
    supervisor::{Instance, Supervisor},
    types::{Height, LightBlock, PeerId, Time, TrustThreshold},
};

use std::collections::HashMap;
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use tendermint_light_client::tests::{AnonLightBlock, TestBisection};

const TEST_FILES_PATH: &str = "./tests/support/";

fn read_json_fixture(file: impl AsRef<Path>) -> String {
    fs::read_to_string(file).unwrap()
}

fn load_multi_peer_testcases(dir: &str) -> Vec<TestBisection<LightBlock>>{
    let paths = fs::read_dir(PathBuf::from(TEST_FILES_PATH).join(dir)).unwrap();
    paths
        .into_iter()
        .flatten()
        .map(|entry| read_json_fixture(entry.path()).to_string())
        .map(|contents| serde_json::from_str::<TestBisection<AnonLightBlock>>(&contents).unwrap())
        .map(|testcase| testcase.into())
        .collect::<Vec<TestBisection<LightBlock>>>();
}

#[derive(Clone)]
struct MockIo {
    chain_id: String,
    light_blocks: HashMap<Height, LightBlock>,
    latest_height: Height,
}

impl MockIo {
    fn new(chain_id: String, light_blocks: Vec<LightBlock>) -> Self {
        let latest_height = light_blocks.iter().map(|lb| lb.height()).max().unwrap();

        let light_blocks = light_blocks
            .into_iter()
            .map(|lb| (lb.height(), lb))
            .collect();

        Self {
            chain_id,
            light_blocks,
            latest_height,
        }
    }
}

#[contract_trait]
impl Io for MockIo {
    fn fetch_light_block(&self, _peer: PeerId, height: AtHeight) -> Result<LightBlock, IoError> {
        let height = match height {
            AtHeight::Highest => self.latest_height,
            AtHeight::At(height) => height,
        };

        self.light_blocks
            .get(&height)
            .cloned()
            .ok_or_else(|| rpc::Error::new((-32600).into(), None).into())
    }
}

fn make_instance(
    peer_id: PeerId,
    trust_options: TrustOptions,
    provider: WitnessProvider<LightBlock>,
) -> Instance {
    let timeout = Duration::from_secs(10);

    let io = MockIo::new(); // TODO

    // TODO: pick light block at height trust_options.height from the light blocks in the provider.
    let trusted_state = todo!();

    let mut light_store = MemoryStore::new(); // TODO
    light_store.insert(trusted_state, VerifiedStatus::Verified);

    let state = State {
        light_store: Box::new(light_store),
        verification_trace: HashMap::new(),
    };

    let options = light_client::Options {
        trust_threshold: TrustThreshold {
            numerator: 1,
            denominator: 3,
        },
        trusting_period: Duration::from_secs(36000),
        clock_drift: Duration::from_secs(1),
        now: Time::now(),
    };

    let verifier = ProdVerifier::default();
    let clock = SystemClock;
    let scheduler = scheduler::basic_bisecting_schedule;

    let light_client = LightClient::new(peer_id, options, clock, scheduler, verifier, io);

    Instance::new(light_client, state)
}

fn run_multipeer_test(tc: TestBisection<LightBlock>) {
    let primary: PeerId = "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE".parse().unwrap();
    let primary_instance = make_instance(primary, &opts);

    //- - - - - - - - - -

    let mut peer_list = PeerList::builder();
    peer_list.primary(primary, primary_instance);

    for provider in tc.witnesses {
        let peer_id = todo!(); // TODO: Make up a peer id
        let instance = make_instance(peer_id, trust_options, provider);
        peer_list.witness(peer_id, instance);
    }

    //- - - - - - - - - -

    let mut supervisor = Supervisor::new(
        peer_list,
        ProdForkDetector::default(),
        ProdEvidenceReporter::new(peer_addr), // TODO: Use mock EvidenceReporter
    );

    // TODO: Add method to `Handle` to get a copy of the current peer list

    let mut handle = supervisor.handle();
    std::thread::spawn(|| supervisor.run());

    let verdict = handle.verify_to_target(target_height);

    // TODO: Check the verdict
    // TODO: Check the peer list
    // TODO: Check we recorded a fork evidence (or not)
}

#[test]
fn deserialize_multi_peer_json() {
    load_multi_peer_testcases("bisection/multi_peer");
}

#[test]
fn run_tests() {
    let testcases = load_multi_peer_testcases("bisection/multi_peer");
    for testcase in testcases {
        run_multipeer_test(testcase);
    }
}
