use tendermint::block::Height;
use tendermint::evidence::Evidence;
use tendermint::hash::Hash;
use tendermint_light_client::errors::Error;
use tendermint_light_client::instance::Instance;
use tendermint_light_client::light_client::TargetOrLatest;
use tendermint_light_client::state::State;
use tendermint_light_client::store::memory::MemoryStore;
use tendermint_light_client::verifier::types::LightBlock;
use tendermint_rpc::{Client, Error as RpcError, HttpClient};

#[derive(Debug)]
pub struct Provider {
    chain_id: String,
    instance: Instance,
    rpc_client: HttpClient,
}

impl Provider {
    pub fn new(chain_id: String, instance: Instance, rpc_client: HttpClient) -> Self {
        Self {
            chain_id,
            instance,
            rpc_client,
        }
    }

    pub fn chain_id(&self) -> &str {
        &self.chain_id
    }

    pub fn peer_id(&self) -> &tendermint::node::Id {
        self.instance.peer_id()
    }

    pub async fn report_evidence(&self, evidence: Evidence) -> Result<Hash, RpcError> {
        self.rpc_client
            .broadcast_evidence(evidence)
            .await
            .map(|response| response.hash)
    }

    pub fn fetch_light_block(&self, height: Height) -> Result<LightBlock, Error> {
        let mut state = State::new(MemoryStore::new());

        self.instance
            .light_client
            .get_or_fetch_block(height, &mut state)
            .map(|(lb, _)| lb)
    }

    pub fn verify_to_highest(&mut self) -> Result<LightBlock, Error> {
        self.instance
            .light_client
            .verify_to_highest(&mut self.instance.state)
    }

    pub fn verify_to_height(&mut self, height: Height) -> Result<LightBlock, Error> {
        self.instance
            .light_client
            .verify_to_target(height, &mut self.instance.state)
    }

    pub fn verify_to_height_with_state(
        &self,
        height: Height,
        state: &mut State,
    ) -> Result<LightBlock, Error> {
        self.instance.light_client.verify_to_target(height, state)
    }

    pub fn get_target_block_or_latest(&mut self, height: Height) -> Result<TargetOrLatest, Error> {
        self.instance
            .light_client
            .get_target_block_or_latest(height, &mut self.instance.state)
    }

    pub fn get_trace(&self, height: Height) -> Vec<LightBlock> {
        self.instance.state.get_trace(height)
    }

    pub fn latest_trusted(&self) -> Option<LightBlock> {
        self.instance.latest_trusted()
    }
}
