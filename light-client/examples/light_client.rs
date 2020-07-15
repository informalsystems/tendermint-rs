use std::collections::HashMap;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use gumdrop::Options;

use tendermint_light_client::supervisor::{Handle as _, Instance, Supervisor};
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
    types::{Height, PeerId, Status, TrustThreshold},
};

#[derive(Debug, Options)]
struct CliOptions {
    #[options(help = "print this help message")]
    help: bool,
    #[options(help = "enable verbose output")]
    verbose: bool,

    #[options(command)]
    command: Option<Command>,
}

#[derive(Debug, Options)]
enum Command {
    #[options(help = "run the light client and continuously sync up to the latest block")]
    Sync(SyncOpts),
}

#[derive(Debug, Options)]
struct SyncOpts {
    #[options(help = "show help for this command")]
    help: bool,
    #[options(
        help = "address of the Tendermint node to connect to",
        meta = "ADDR",
        default = "tcp://127.0.0.1:26657"
    )]
    address: tendermint::net::Address,
    #[options(
        help = "height of the initial trusted state (optional if store already initialized)",
        meta = "HEIGHT"
    )]
    trusted_height: Option<Height>,
    #[options(
        help = "path to the database folder",
        meta = "PATH",
        default = "./lightstore"
    )]
    db_path: PathBuf,
}

#[tokio::main]
async fn main() {
    let opts = CliOptions::parse_args_default_or_exit();
    match opts.command {
        None => {
            eprintln!("Please specify a command:");
            eprintln!("{}\n", CliOptions::command_list().unwrap());
            eprintln!("{}\n", CliOptions::usage());
            std::process::exit(1);
        }
        Some(Command::Sync(sync_opts)) => sync_cmd(sync_opts).await,
    }
}

async fn make_instance(
    peer_id: PeerId,
    addr: tendermint::net::Address,
    db_path: impl AsRef<Path>,
    opts: &SyncOpts,
) -> Instance {
    let mut peer_map = HashMap::new();
    peer_map.insert(peer_id, addr);

    let timeout = Duration::from_secs(10);
    let io = ProdIo::new(peer_map, Some(timeout));

    let db = sled::open(db_path).unwrap_or_else(|e| {
        println!("[ error ] could not open database: {}", e);
        std::process::exit(1);
    });

    let mut light_store = SledStore::new(db);

    if let Some(height) = opts.trusted_height {
        let trusted_state = io
            .fetch_light_block(peer_id, AtHeight::At(height))
            .await
            .unwrap_or_else(|e| {
                println!("[ error ] could not retrieve trusted header: {}", e);
                std::process::exit(1);
            });

        light_store.insert(trusted_state, Status::Verified);
    } else if light_store.latest(Status::Verified).is_none() {
        println!("[ error ] no trusted state in database, please specify a trusted header");
        std::process::exit(1);
    }

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
    };

    let verifier = ProdVerifier::default();
    let clock = SystemClock;
    let scheduler = scheduler::basic_bisecting_schedule;

    let light_client = LightClient::new(peer_id, options, clock, scheduler, verifier, io);

    Instance::new(light_client, state)
}

async fn sync_cmd(opts: SyncOpts) {
    let addr = opts.address.clone();

    let primary: PeerId = "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE".parse().unwrap();
    let witness: PeerId = "CEFEEDBADFADAD0C0CEEFACADE0ADEADBEEFC0FF".parse().unwrap();

    let primary_path = opts.db_path.join(primary.to_string());
    let witness_path = opts.db_path.join(witness.to_string());

    let primary_instance = make_instance(primary, addr.clone(), primary_path, &opts).await;
    let witness_instance = make_instance(witness, addr.clone(), witness_path, &opts).await;

    let mut peer_addr = HashMap::new();
    peer_addr.insert(primary, addr.clone());
    peer_addr.insert(witness, addr);

    let peer_list = PeerList::builder()
        .primary(primary, primary_instance)
        .witness(witness, witness_instance)
        .build();

    let mut supervisor = Supervisor::new(
        peer_list,
        ProdForkDetector::default(),
        ProdEvidenceReporter::new(peer_addr),
    );

    let handle = supervisor.handle();

    std::thread::spawn(|| supervisor.run());

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
}
