#![allow(unused)]

use std::{convert::Infallible, str::FromStr, time::Duration};

use tendermint::{evidence::Evidence, Time};
use tendermint_light_client::{
    builder::LightClientBuilder,
    instance::Instance,
    light_client::Options,
    misbehavior::{detect_divergence, Provider},
    store::memory::MemoryStore,
    types::{Hash, Height, LightBlock, TrustThreshold},
};
use tendermint_rpc::{Client, HttpClient};

use clap::Parser;
use color_eyre::eyre::{eyre, Result};
use futures::future::join_all;
use tracing::{error, info, warn};
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

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
    /// Identifier of the chain
    chain_id: String,

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

    /// Height of the header to verify
    #[clap(long)]
    height: Option<Height>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(EnvFilter::new(format!(
            "tendermint_light_client={rust_log},light_client_cli={rust_log}",
        )))
        .finish()
        .init();

    let args = Cli::parse();

    let mut primary = make_provider(
        &args.chain_id,
        &args.primary,
        args.trusted_height,
        args.trusted_hash,
    )
    .await?;

    let trusted_block = primary
        .latest_trusted()
        .ok_or_else(|| eyre!("No trusted state found for primary"))?;

    let primary_block = if let Some(target_height) = args.height {
        info!("Verifying to height {} on primary...", target_height);
        primary.verify_to_height(target_height)
    } else {
        info!("Verifying to latest height on primary...");
        primary.verify_to_highest()
    }?;

    info!("Verified to height {} on primary", primary_block.height());
    let primary_trace = primary.get_trace(primary_block.height());

    let witnesses = join_all(args.witnesses.0.iter().map(|addr| {
        make_provider(
            &args.chain_id,
            addr,
            trusted_block.height(),
            trusted_block.signed_header.header.hash(),
        )
    }))
    .await;

    let mut witnesses = witnesses.into_iter().collect::<Result<Vec<_>>>()?;

    info!(
        "Running misbehavior detection against {} witnesses...",
        witnesses.len()
    );

    // FIXME
    let max_clock_drift = Duration::from_secs(1);
    let max_block_lag = Duration::from_secs(1);
    let now = Time::now();

    let result = detect_divergence(
        &mut primary,
        witnesses.as_mut_slice(),
        primary_trace,
        max_clock_drift,
        max_block_lag,
        now,
    )
    .await;

    use tendermint_light_client::misbehavior::error::divergence::ErrorDetail;

    match result {
        Ok(()) => {
            info!("No misbehavior detected");
        },
        Err(e) => match e.detail() {
            ErrorDetail::Divergence(_) => {
                warn!("Found divergence! See log above for details.");
            },
            ErrorDetail::NoWitnesses(_) => {
                error!("No witnesses provided");
            },
            ErrorDetail::FailedHeaderCrossReferencing(e) => {
                error!("Failed to cross-reference headers: {}", e)
            },
            ErrorDetail::TraceTooShort(_) => {
                error!("Found a trace that is too short to detect misbehavior")
            },
        },
    }

    Ok(())
}

async fn make_provider(
    chain_id: &str,
    rpc_addr: &str,
    trusted_height: Height,
    trusted_hash: Hash,
) -> Result<Provider> {
    use tendermint_rpc::client::CompatMode;

    let rpc_client = HttpClient::builder(rpc_addr.parse().unwrap())
        .compat_mode(CompatMode::V0_34)
        .build()?;

    let node_id = rpc_client.status().await?.node_info.id;
    let light_store = Box::new(MemoryStore::new());
    let options = Options {
        trust_threshold: TrustThreshold::TWO_THIRDS,
        trusting_period: Duration::from_secs(60 * 60 * 24 * 14),
        clock_drift: Duration::from_secs(60),
    };

    let instance =
        LightClientBuilder::prod(node_id, rpc_client.clone(), light_store, options, None)
            .trust_primary_at(trusted_height, trusted_hash)?
            .build();

    Ok(Provider::new(chain_id.to_string(), instance, rpc_client))
}
