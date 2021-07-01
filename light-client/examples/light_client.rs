use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use gumdrop::Options;

use tendermint::Hash;
use tendermint_rpc as rpc;

use tendermint_light_client::supervisor::{Handle as _, Instance};
use tendermint_light_client::{
    builder::{LightClientBuilder, SupervisorBuilder},
    light_client,
    store::sled::SledStore,
    types::{Height, PeerId, TrustThreshold},
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
    address: tendermint_rpc::Url,
    #[options(
        help = "height of the initial trusted state (optional if store already initialized)",
        meta = "HEIGHT"
    )]
    trusted_height: Option<Height>,
    #[options(
        help = "hash of the initial trusted state (optional if store already initialized)",
        meta = "HASH"
    )]
    trusted_hash: Option<Hash>,
    #[options(
        help = "path to the database folder",
        meta = "PATH",
        default = "./lightstore"
    )]
    db_path: PathBuf,
}

fn main() {
    let opts = CliOptions::parse_args_default_or_exit();

    match opts.command {
        None => {
            eprintln!("Please specify a command:");
            eprintln!("{}\n", CliOptions::command_list().unwrap());
            eprintln!("{}\n", CliOptions::usage());
            std::process::exit(1);
        }
        Some(Command::Sync(sync_opts)) => sync_cmd(sync_opts).unwrap_or_else(|e| {
            eprintln!("Command failed: {}", e);
            std::process::exit(1);
        }),
    }
}

fn make_instance(
    peer_id: PeerId,
    addr: tendermint_rpc::Url,
    db_path: impl AsRef<Path>,
    opts: &SyncOpts,
) -> Result<Instance, Box<dyn std::error::Error>> {
    let light_store = SledStore::open(db_path)?;
    let rpc_client = rpc::HttpClient::new(addr).unwrap();
    let options = light_client::Options {
        trust_threshold: TrustThreshold::default(),
        trusting_period: Duration::from_secs(36000),
        clock_drift: Duration::from_secs(1),
    };

    let builder =
        LightClientBuilder::prod(peer_id, rpc_client, Box::new(light_store), options, None);

    let builder = if let (Some(height), Some(hash)) = (opts.trusted_height, opts.trusted_hash) {
        builder.trust_primary_at(height, hash)
    } else {
        builder.trust_from_store()
    }?;

    Ok(builder.build())
}

fn sync_cmd(opts: SyncOpts) -> Result<(), Box<dyn std::error::Error>> {
    let primary: PeerId = "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE".parse().unwrap();
    let witness: PeerId = "CEFEEDBADFADAD0C0CEEFACADE0ADEADBEEFC0FF".parse().unwrap();

    let primary_addr = opts.address.clone();
    let witness_addr = opts.address.clone();

    let primary_path = opts.db_path.join(primary.to_string());
    let witness_path = opts.db_path.join(witness.to_string());

    let primary_instance = make_instance(primary, primary_addr.clone(), primary_path, &opts)?;
    let witness_instance = make_instance(witness, witness_addr.clone(), witness_path, &opts)?;

    let supervisor = SupervisorBuilder::new()
        .primary(primary, primary_addr, primary_instance)
        .witness(witness, witness_addr, witness_instance)
        .build_prod();

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
