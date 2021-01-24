//! `intialize` subcommand

use std::ops::Deref;
use std::time::Duration;

use crate::application::app_config;
use crate::config::LightClientConfig;
use crate::config::LightNodeConfig;

use abscissa_core::status_err;
use abscissa_core::status_warn;
use abscissa_core::Command;
use abscissa_core::Options;
use abscissa_core::Runnable;

use tendermint::{hash, Hash};

use std::convert::TryInto;
use tendermint_light_client::builder::LightClientBuilder;
use tendermint_light_client::store::sled::SledStore;
use tendermint_light_client::store::LightStore;
use tendermint_light_client::supervisor::Instance;
use tendermint_light_client::types::Height;

use tendermint_rpc as rpc;

/// `initialize` subcommand
#[derive(Command, Debug, Default, Options)]
pub struct InitCmd {
    #[options(
        free,
        help = "subjective height of the initial trusted state to initialize the node with"
    )]
    pub height: u64,

    #[options(
        free,
        help = "hash of the initial subjectively trusted header to initialize the node with"
    )]
    pub header_hash: String,
}

impl Runnable for InitCmd {
    fn run(&self) {
        let subjective_header_hash =
            Hash::from_hex_upper(hash::Algorithm::Sha256, &self.header_hash).unwrap();

        let node_config = app_config().deref().clone();
        let light_client_config = node_config.light_clients.first().unwrap();

        if let Err(e) = initialize_subjectively(
            self.height.try_into().unwrap(),
            subjective_header_hash,
            &node_config,
            &light_client_config,
            Some(node_config.rpc_config.request_timeout),
        ) {
            status_err!("failed to initialize light client: {}", e);
            // TODO: Set exit code to 1
        }
    }
}

fn initialize_subjectively(
    height: Height,
    subjective_header_hash: Hash,
    node_config: &LightNodeConfig,
    config: &LightClientConfig,
    timeout: Option<Duration>,
) -> Result<Instance, String> {
    let light_store =
        SledStore::open(&config.db_path).map_err(|e| format!("could not open database: {}", e))?;

    if let Some(trusted_state) = light_store.highest_trusted_or_verified() {
        status_warn!(
            "already existing trusted or verified state of height {} in database: {:?}",
            trusted_state.signed_header.header.height,
            config.db_path
        );
    }

    let rpc_client = rpc::HttpClient::new(config.address.clone()).map_err(|e| e.to_string())?;

    let builder = LightClientBuilder::prod(
        config.peer_id,
        rpc_client,
        Box::new(light_store),
        node_config.clone().into(),
        timeout,
    );

    let builder = builder
        .trust_primary_at(height, subjective_header_hash)
        .map_err(|e| {
            format!(
                "could not trust header at height {} and hash {}. Reason: {}",
                height, subjective_header_hash, e
            )
        })?;

    Ok(builder.build())
}
