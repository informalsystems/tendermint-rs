//! `start` subcommand - start the light node.

/// App-local prelude includes `app_reader()`/`app_writer()`/`app_config()`
/// accessors along with logging macros. Customize as you see fit.
use abscissa_core::{config, Command, FrameworkError, Options, Runnable};
use std::process;

use crate::application::{app_config, APPLICATION};
use crate::config::{LightClientConfig, LightNodeConfig};
use crate::rpc::Server;

use crate::rpc;
use abscissa_core::path::PathBuf;
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
use tendermint_light_client::types::Status;

/// `start` subcommand
///
#[derive(Command, Debug, Options)]
pub struct StartCmd {
    /// Path to configuration file
    #[options(
        short = "l",
        long = "listen",
        help = "address the rpc server will serve"
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

            let mut supervisor = Supervisor::new(
                peer_list,
                ProdForkDetector::default(),
                ProdEvidenceReporter::new(peer_map.clone()),
            );

            let handle = supervisor.handle();
            let rpc_handler = supervisor.handle();
            let server = Server::new(rpc_handler);

            std::thread::spawn(|| supervisor.run());
            let laddr = app_config().rpc_config.listen_addr;
            // TODO: figure out howto handle the potenial error on run
            std::thread::spawn(move || rpc::run(server, &laddr.to_string()));

            loop {
                match handle.verify_to_highest() {
                    Ok(light_block) => {
                        println!("[info] synced to block {}", light_block.height());
                    }
                    Err(err) => {
                        println!("[error] sync failed: {}", err);
                    }
                }

                std::thread::sleep(Duration::from_millis(800));
            }
        }) {
            eprintln!("Error while running application: {}", err);
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
        // Todo figure out if other options would be reasonable to overwrite via CLI arguments.
        if let Some(addr) = self.listen_addr {
            config.rpc_config.listen_addr = addr;
        }
        Ok(config)
    }
}
impl StartCmd {
    fn make_instance(
        &self,
        light_config: &LightClientConfig,
        io: ProdIo,
        options: light_client::Options,
    ) -> Instance {
        let peer_id = light_config.peer_id;
        let db_path = light_config.db_path.clone();

        let db = sled::open(db_path).unwrap_or_else(|e| {
            println!("[ error ] could not open database: {}", e);
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
}

impl StartCmd {
    fn assert_init_was_run() {
        // TODO: handle errors properly:
        let primary_db_path = app_config().light_clients.first().unwrap().db_path.clone();
        let db = sled::open(primary_db_path).unwrap_or_else(|e| {
            println!("[error] could not open database: {}", e);
            std::process::exit(1);
        });

        let primary_store = SledStore::new(db);

        if primary_store.latest(Status::Verified).is_none() {
            println!("[error] no trusted state in store for primary, please initialize with the `initialize` subcommand first");
            std::process::exit(1);
        }
    }
}
