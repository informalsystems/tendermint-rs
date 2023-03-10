use std::{convert::Infallible, str::FromStr, time::Duration};

use tendermint_light_client::{
    builder::LightClientBuilder,
    instance::Instance,
    light_client::Options,
    misbehavior::handle_conflicting_headers,
    store::memory::MemoryStore,
    types::{Hash, Height, TrustThreshold},
};

use clap::Parser;
use color_eyre::eyre::{eyre, Result};
use tendermint_rpc::{Client, HttpClient};
use tracing::{error, info, warn};
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Clone, Debug)]
struct List<T>(Vec<T>);

impl FromStr for List<String> {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.split(',').map(|s| s.to_string()).collect()))
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Chain moniker
    chain: String,

    /// Primary RPC address
    #[clap(short, long)]
    primary: String,

    /// Comma-separated list of witnesses RPC addresses
    #[clap(short, long)]
    witnesses: List<String>,

    /// Height of trusted header
    #[clap(long)]
    trusted_height: Height,

    /// Hash of trusted header
    #[clap(long)]
    trusted_hash: Hash,

    /// Height of target header
    #[clap(long)]
    height: Height,
}

async fn create_light_client(
    rpc_addr: &str,
    trusted_height: Height,
    trusted_hash: Hash,
) -> Result<Instance> {
    let rpc_client = HttpClient::new(rpc_addr)?;
    let node_id = rpc_client.status().await?.node_info.id;
    let light_store = Box::new(MemoryStore::new());
    let options = Options {
        trust_threshold: TrustThreshold::TWO_THIRDS,
        trusting_period: Duration::from_secs(60 * 60 * 24 * 14),
        clock_drift: Duration::from_secs(60),
    };

    let instance = LightClientBuilder::prod(node_id, rpc_client, light_store, options, None)
        .trust_primary_at(trusted_height, trusted_hash)?
        .build();

    Ok(instance)
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().with_target(false).finish().init();

    let args = Cli::parse();

    let mut primary =
        create_light_client(&args.primary, args.trusted_height, args.trusted_hash).await?;
    let trusted_block = primary
        .latest_trusted()
        .ok_or_else(|| eyre!("No trusted state found for primary"))?;

    info!("Verifying to height {} on primary...", args.height);

    let primary_block = primary
        .light_client
        .verify_to_target(args.height, &mut primary.state)?;

    let primary_hash = primary_block.signed_header.header.hash();

    for witness_addr in args.witnesses.0 {
        let mut witness =
            create_light_client(&witness_addr, args.trusted_height, args.trusted_hash).await?;

        let witness_block = witness
            .light_client
            .verify_to_target(args.height, &mut witness.state)?;

        let witness_hash = witness_block.signed_header.header.hash();

        if primary_hash != witness_hash {
            warn!(
                "Hash mismatch between primary and witness: {} != {}",
                primary_hash, witness_hash
            );

            info!("Performing misbehavior detection against witness '{witness_addr}'...");

            let attack = handle_conflicting_headers(
                &witness,
                &primary_block,
                &trusted_block,
                &witness_block,
            )?;

            if let Some(attack) = attack {
                use tendermint::evidence::Evidence;
                let evidence = Evidence::from(attack);
                let json = serde_json::to_string_pretty(&evidence)?;
                warn!("Evidence found:\n{}", json);
            } else {
                error!("No evidence found");
            }
        }
    }

    Ok(())
}
