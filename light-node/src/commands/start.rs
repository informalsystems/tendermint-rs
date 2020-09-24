//! `start` subcommand - start the light node.

use std::process;

use crate::application::{app_config, APPLICATION};
use crate::config::{LightClientConfig, LightNodeConfig};
use crate::rpc;
use crate::rpc::Server;

use abscissa_core::config;
use abscissa_core::path::PathBuf;
use abscissa_core::status_err;
use abscissa_core::status_info;
use abscissa_core::Command;
use abscissa_core::FrameworkError;
use abscissa_core::Options;
use abscissa_core::Runnable;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Deref;
use std::time::Duration;

use tendermint_light_client::builder::LightClientBuilder;
use tendermint_light_client::evidence::ProdEvidenceReporter;
use tendermint_light_client::fork_detector::ProdForkDetector;
use tendermint_light_client::light_client;
use tendermint_light_client::peer_list::{PeerList, PeerListBuilder};
use tendermint_light_client::store::sled::SledStore;
use tendermint_light_client::store::LightStore;
use tendermint_light_client::supervisor::{Handle, Instance, Supervisor};

/// `start` subcommand
#[derive(Command, Debug, Options)]
pub struct StartCmd {
    /// Path to configuration file
    #[options(
        short = "b",
        long = "jsonrpc-server-addr",
        help = "address the rpc server will bind to"
    )]
    pub listen_addr: Option<SocketAddr>,

    /// Path to configuration file
    #[options(short = "c", long = "config", help = "path to light_node.toml")]
    pub config: Option<PathBuf>,
}

impl Runnable for StartCmd {
    /// Start the application.
    fn run(&self) {
        if let Err(err) = abscissa_tokio::run(&APPLICATION, async {
            if let Err(e) = StartCmd::assert_init_was_run() {
                status_err!(&e);
                panic!(e);
            }

            let supervisor = match self.construct_supervisor() {
                Ok(supervisor) => supervisor,
                Err(e) => {
                    status_err!(&e);
                    panic!(e);
                }
            };

            let rpc_handler = supervisor.handle();
            StartCmd::start_rpc_server(rpc_handler);

            let handle = supervisor.handle();
            std::thread::spawn(|| supervisor.run());

            loop {
                match handle.verify_to_highest() {
                    Ok(light_block) => {
                        status_info!("synced to block:", light_block.height().to_string());
                    }
                    Err(err) => {
                        status_err!("sync failed: {}", err);
                    }
                }

                // TODO(liamsi): use ticks and make this configurable:
                std::thread::sleep(Duration::from_millis(800));
            }
        }) {
            status_err!("Unexpected error while running application: {}", err);
            process::exit(1);
        }
    }
}

impl config::Override<LightNodeConfig> for StartCmd {
    // Process the given command line options, overriding settings from
    // a configuration file using explicit flags taken from command-line
    // arguments.
    fn override_config(
        &self,
        mut config: LightNodeConfig,
    ) -> Result<LightNodeConfig, FrameworkError> {
        // TODO(liamsi): figure out if other options would be reasonable to overwrite via CLI
        // arguments.
        if let Some(addr) = self.listen_addr {
            config.rpc_config.listen_addr = addr;
        }
        Ok(config)
    }
}

impl StartCmd {
    fn assert_init_was_run() -> Result<(), String> {
        let db_path = app_config().light_clients.first().unwrap().db_path.clone();
        let db = sled::open(db_path).map_err(|e| format!("could not open database: {}", e))?;

        let primary_store = SledStore::new(db);
        if primary_store.latest_trusted_or_verified().is_none() {
            return Err("no trusted or verified state in store for primary, please initialize with the `initialize` subcommand first".to_string());
        }

        Ok(())
    }

    fn start_rpc_server<H>(h: H)
    where
        H: Handle + Send + Sync + 'static,
    {
        let server = Server::new(h);
        let laddr = app_config().rpc_config.listen_addr;
        // TODO(liamsi): figure out how to handle the potential error on run
        std::thread::spawn(move || rpc::run(server, &laddr.to_string()));
        status_info!("started RPC server:", laddr.to_string());
    }

    fn make_instance(
        &self,
        light_config: &LightClientConfig,
        options: light_client::Options,
        timeout: Option<Duration>,
    ) -> Result<Instance, String> {
        let rpc_client = tendermint_rpc::HttpClient::new(light_config.address.clone())
            .map_err(|e| format!("failed to create HTTP client: {}", e))?;

        let db_path = light_config.db_path.clone();
        let db = sled::open(db_path).map_err(|e| format!("could not open database: {}", e))?;

        let light_store = SledStore::new(db);

        let builder = LightClientBuilder::prod(
            light_config.peer_id,
            rpc_client,
            Box::new(light_store),
            options,
            timeout,
        );

        let builder = builder
            .trust_from_store()
            .map_err(|e| format!("could not set initial trusted state: {}", e))?;

        Ok(builder.build())
    }

    fn construct_supervisor(&self) -> Result<Supervisor, String> {
        let conf = app_config().deref().clone();
        let options: light_client::Options = conf.into();

        let timeout = app_config().rpc_config.request_timeout;

        let mut peer_list: PeerListBuilder<Instance> = PeerList::builder();
        for (i, light_conf) in app_config().light_clients.iter().enumerate() {
            let instance = self.make_instance(light_conf, options, Some(timeout))?;

            if i == 0 {
                // primary instance
                peer_list = peer_list.primary(instance.light_client.peer, instance);
            } else {
                peer_list = peer_list.witness(instance.light_client.peer, instance);
            }
        }

        let peer_list = peer_list.build();

        let peer_map: HashMap<_, _> = app_config()
            .light_clients
            .iter()
            .map(|lc| (lc.peer_id, lc.address.clone()))
            .collect();

        Ok(Supervisor::new(
            peer_list,
            ProdForkDetector::default(),
            ProdEvidenceReporter::new(peer_map),
        ))
    }
}
