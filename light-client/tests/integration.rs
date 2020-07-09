//! Light Client integration tests.
//!
//! These are all ignored by default, since they test against running
//! `tendermint node --proxy_app=kvstore`. They can be run using:
//!
//! ```
//! cargo test -- --ignored
//! ```

use tendermint_light_client::{
    components::{
        clock::SystemClock,
        io::{AtHeight, Io, IoError, ProdIo},
        scheduler,
        verifier::ProdVerifier,
    },
    evidence::{ConflictingHeadersEvidence, Evidence, EvidenceReporter},
    fork_detector::ProdForkDetector,
    light_client::{self, LightClient},
    operations::hasher::ProdHasher,
    peer_list::PeerList,
    state::State,
    store::{memory::MemoryStore, LightStore},
    supervisor::{Handle, Instance, Supervisor},
    types::{LightBlock, PeerId, Status, TMLightBlock, TMSignedHeader, TrustThreshold},
};

use tendermint::abci::transaction::Hash as TransactionHash;

use std::collections::HashMap;
use std::time::Duration;

fn make_instance(
    peer_id: PeerId,
    options: light_client::Options,
    io: ProdIo,
) -> Instance<TMLightBlock> {
    let trusted_state = io
        .fetch_light_block(peer_id, AtHeight::Highest)
        .expect("could not request latest light block");

    let mut light_store = MemoryStore::new();
    light_store.insert(trusted_state, Status::Trusted);

    let state = State {
        light_store: Box::new(light_store),
        verification_trace: HashMap::new(),
    };

    let verifier = ProdVerifier::default();
    let clock = SystemClock;
    let scheduler = scheduler::basic_bisecting_schedule;

    let light_client = LightClient::new(peer_id, options, clock, scheduler, verifier, io);

    Instance::new(light_client, state)
}

struct TestEvidenceReporter;

#[contracts::contract_trait]
impl EvidenceReporter<TMLightBlock> for TestEvidenceReporter {
    fn report(&self, evidence: Evidence, peer: PeerId) -> Result<TransactionHash, IoError> {
        panic!(
            "unexpected fork detected for peer {} with evidence: {:?}",
            peer, evidence
        );
    }

    fn build_conflicting_headers_evidence(
        &self,
        sh1: TMSignedHeader,
        sh2: TMSignedHeader,
    ) -> Evidence {
        Evidence::ConflictingHeaders(Box::new(ConflictingHeadersEvidence::new(sh1, sh2)))
    }
}

#[test]
#[ignore]
fn sync() {
    let primary: PeerId = "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE".parse().unwrap();
    let witness: PeerId = "CEFEEDBADFADAD0C0CEEFACADE0ADEADBEEFC0FF".parse().unwrap();

    let node_address: tendermint::net::Address = "tcp://127.0.0.1:26657".parse().unwrap();

    // Because our CI infrastructure can only spawn a single Tendermint node at the moment,
    // we run this test against this very node as both the primary and witness.
    // In a production environment, one should make sure that the primary and witness are
    // different nodes, and check that the configured peer IDs match the ones returned
    // by the nodes.
    let mut peer_map = HashMap::new();
    peer_map.insert(primary, node_address.clone());
    peer_map.insert(witness, node_address);

    let io = ProdIo::new(peer_map, Some(Duration::from_secs(2)));

    let options = light_client::Options {
        trust_threshold: TrustThreshold {
            numerator: 1,
            denominator: 3,
        },
        trusting_period: Duration::from_secs(60 * 60), // 60 minutes
        clock_drift: Duration::from_secs(5 * 60),      // 5 minutes
    };

    let primary_instance = make_instance(primary, options, io.clone());
    let witness_instance = make_instance(witness, options, io);

    let peer_list = PeerList::builder()
        .primary(primary, primary_instance)
        .witness(witness, witness_instance)
        .build();

    let mut supervisor = Supervisor::new(
        peer_list,
        ProdHasher::default(),
        ProdForkDetector::default(),
        TestEvidenceReporter,
    );

    let handle = supervisor.handle();
    std::thread::spawn(|| supervisor.run());

    let max_iterations: usize = 20;

    for i in 1..=max_iterations {
        println!("[info ] - iteration {}/{}", i, max_iterations);

        match handle.verify_to_highest() {
            Ok(light_block) => {
                println!("[info ] synced to block {}", light_block.height());
            }
            Err(err) => {
                println!("[error] sync failed: {}", err);
                panic!("failed to sync to highest: {}", err);
            }
        }

        std::thread::sleep(Duration::from_millis(800));
    }
}
