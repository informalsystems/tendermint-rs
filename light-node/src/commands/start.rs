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

use tendermint_light_client::components::clock::SystemClock;
use tendermint_light_client::components::io::ProdIo;
use tendermint_light_client::components::scheduler;
use tendermint_light_client::components::verifier::ProdVerifier;
use tendermint_light_client::evidence::ProdEvidenceReporter;
use tendermint_light_client::fork_detector::ProdForkDetector;
use tendermint_light_client::light_client;
use tendermint_light_client::light_client::LightClient;
use tendermint_light_client::peer_list::{PeerList, PeerListBuilder};
use tendermint_light_client::state::State;
use tendermint_light_client::store::sled::SledStore;
use tendermint_light_client::store::LightStore;
use tendermint_light_client::supervisor::Handle;
use tendermint_light_client::supervisor::{Instance, Supervisor};

/// `start` subcommand
///
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
            StartCmd::assert_init_was_run();
            let mut supervisor = self.construct_supervisor();

            let rpc_handler = supervisor.handle();
            StartCmd::start_rpc_server(rpc_handler);

            let handle = supervisor.handle();
            std::thread::spawn(|| supervisor.run());

            loop {
                match handle.verify_to_highest() {
                    Ok(light_block) => {
                        status_info!("synced to block {}", light_block.height().to_string());
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
        // TODO(liamsi): figure out if other options would be reasonable to overwrite via CLI arguments.
        if let Some(addr) = self.listen_addr {
            config.rpc_config.listen_addr = addr;
        }
        Ok(config)
    }
}
impl StartCmd {
    fn assert_init_was_run() {
        // TODO(liamsi): handle errors properly:
        let primary_db_path = app_config().light_clients.first().unwrap().db_path.clone();
        let db = sled::open(primary_db_path).unwrap_or_else(|e| {
            status_err!("could not open database: {}", e);
            std::process::exit(1);
        });

        let primary_store = SledStore::new(db);

        if primary_store.latest_trusted_or_verified().is_none() {
            status_err!("no trusted or verified state in store for primary, please initialize with the `initialize` subcommand first");
            std::process::exit(1);
        }
    }
    // TODO: this should do proper error handling, be gerneralized
    // then moved to to the light-client crate.
    fn make_instance(
        &self,
        light_config: &LightClientConfig,
        io: ProdIo,
        options: light_client::Options,
    ) -> Instance {
        let peer_id = light_config.peer_id;
        let db_path = light_config.db_path.clone();

        let db = sled::open(db_path).unwrap_or_else(|e| {
            status_err!("could not open database: {}", e);
            std::process::exit(1);
        });

        let light_store = SledStore::new(db);

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

    fn start_rpc_server<H>(h: H)
    where
        H: Handle + Send + Sync + 'static,
    {
        let server = Server::new(h);
        let laddr = app_config().rpc_config.listen_addr;
        // TODO(liamsi): figure out how to handle the potential error on run
        std::thread::spawn(move || rpc::run(server, &laddr.to_string()));
    }
}

impl StartCmd {
    fn construct_supervisor(&self) -> Supervisor {
        // TODO(ismail): we need to verify the addr <-> peerId mappings somewhere!
        let mut peer_map = HashMap::new();
        for light_conf in &app_config().light_clients {
            peer_map.insert(light_conf.peer_id, light_conf.address.clone());
        }
        let io = ProdIo::new(
            peer_map.clone(),
            Some(app_config().rpc_config.request_timeout),
        );
        let conf = app_config().deref().clone();
        let options: light_client::Options = conf.into();

        let mut peer_list: PeerListBuilder<Instance> = PeerList::builder();
        for (i, light_conf) in app_config().light_clients.iter().enumerate() {
            let instance = self.make_instance(light_conf, io.clone(), options);
            if i == 0 {
                // primary instance
                peer_list = peer_list.primary(instance.light_client.peer, instance);
            } else {
                peer_list = peer_list.witness(instance.light_client.peer, instance);
            }
        }
        let peer_list = peer_list.build();

        Supervisor::new(
            peer_list,
            ProdForkDetector::default(),
            ProdEvidenceReporter::new(peer_map),
        )
    }
}
