//! Light Client integration tests.
//!
//! If you have a kvstore app running on 127.0.0.1:26657,
//! these can be run using:
//!
//!     cargo test
//!
//! Or else, if you have docker installed, you can tell the tests to run an endpoint,
//! by running:
//!
//!     cargo make
//!
//! (Make sure you install cargo-make using `cargo install cargo-make` first.)

use async_trait::async_trait;

use tendermint_light_client::{
    builder::{LightClientBuilder, SupervisorBuilder},
    components::io::{AtHeight, AsyncIo, IoError, RpcIo},
    errors::Error,
    evidence::{Evidence, EvidenceReporter},
    store::{memory::MemoryStore, LightStore},
    supervisor::{Handle, Instance, Supervisor},
    verifier::{
        options::Options as LightClientOptions,
        types::{Height, PeerId, Status, TrustThreshold},
    },
};
use tendermint_rpc as rpc;

use std::convert::TryFrom;
use std::time::Duration;

struct TestEvidenceReporter;

#[async_trait]
impl EvidenceReporter for TestEvidenceReporter {
    async fn report(
        &self,
        evidence: Evidence,
        peer: PeerId,
    ) -> Result<rpc::abci::transaction::Hash, IoError> {
        panic!(
            "unexpected fork detected for peer {} with evidence: {:?}",
            peer, evidence
        );
    }
}

async fn make_instance(peer_id: PeerId, options: LightClientOptions, address: rpc::Url) -> Instance {
    let rpc_client = rpc::HttpClient::new(address).unwrap();
    let io = RpcIo::new(peer_id, rpc_client.clone(), Some(Duration::from_secs(2)));
    let latest_block = io.fetch_light_block(AtHeight::Highest).await.unwrap();

    let mut light_store = Box::new(MemoryStore::new());
    light_store.insert(latest_block, Status::Trusted);

    LightClientBuilder::prod(
        peer_id,
        rpc_client,
        light_store,
        options,
        Some(Duration::from_secs(2)),
    )
    .trust_from_store()
    .unwrap()
    .build()
}

async fn make_supervisor() -> Supervisor {
    let primary: PeerId = "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE".parse().unwrap();
    let witness: PeerId = "CEFEEDBADFADAD0C0CEEFACADE0ADEADBEEFC0FF".parse().unwrap();

    // Because our CI infrastructure can only spawn a single Tendermint node at the moment,
    // we run this test against this very node as both the primary and witness.
    // In a production environment, one should make sure that the primary and witness are
    // different nodes, and check that the configured peer IDs match the ones returned
    // by the nodes.
    let node_address: rpc::Url = "http://127.0.0.1:26657".parse().unwrap();

    let options = LightClientOptions {
        trust_threshold: TrustThreshold::new(1, 3).unwrap(),
        trusting_period: Duration::from_secs(60 * 60), // 60 minutes
        clock_drift: Duration::from_secs(5 * 60),      // 5 minutes
    };

    let primary_instance = make_instance(primary, options, node_address.clone()).await;
    let witness_instance = make_instance(witness, options, node_address.clone()).await;

    SupervisorBuilder::new()
        .primary(primary, node_address.clone(), primary_instance)
        .witness(witness, node_address, witness_instance)
        .build_prod()
}

#[tokio::test]
async fn forward() -> Result<(), Error> {
    let supervisor = make_supervisor().await;

    let handle = supervisor.handle();
    tokio::spawn(supervisor.run());

    let max_iterations: usize = 10;

    for i in 1..=max_iterations {
        println!("[info ] - iteration {}/{}", i, max_iterations);

        match handle.verify_to_highest().await {
            Ok(light_block) => {
                println!("[info ] synced to block {}", light_block.height());
            }
            Err(err) => {
                println!("[error] sync failed: {}", err);
                panic!("failed to sync to highest: {}", err);
            }
        }

        tokio::time::sleep(Duration::from_millis(800)).await;
    }

    handle.terminate().await
}

#[tokio::test]
async fn backward() -> Result<(), Error> {
    let supervisor = make_supervisor().await;

    let handle = supervisor.handle();
    tokio::spawn(supervisor.run());

    let max_iterations: usize = 10;

    // Sleep a little bit to ensure we have a few blocks already
    tokio::time::sleep(Duration::from_secs(2)).await;

    for i in 1..=max_iterations {
        println!("[info ] - iteration {}/{}", i, max_iterations);

        // First we sync to the highest block to have a high enough trusted state
        let trusted_state = handle.verify_to_highest().await?;
        println!("[info ] synced to highest block {}", trusted_state.height());

        // Then we pick a height below the trusted state
        let target_height = Height::try_from(trusted_state.height().value() / 2).unwrap();

        // We now try to verify a block at this height
        let light_block = handle.verify_to_target(target_height).await?;
        println!("[info ] verified lower block {}", light_block.height());

        tokio::time::sleep(Duration::from_millis(800)).await;
    }

    // NB: If terminate is not called explicitly the test would hang with the supervisor loop
    // dangled in the background.
    handle.terminate().await
}
