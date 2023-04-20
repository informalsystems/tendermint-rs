#![allow(unused)]

use std::{convert::Infallible, str::FromStr, time::Duration};

use tendermint::{evidence::Evidence, Time};
use tendermint_light_client::{
    builder::LightClientBuilder,
    instance::Instance,
    light_client::Options,
    store::memory::MemoryStore,
    types::{Hash, Height, LightBlock, TrustThreshold},
};
use tendermint_light_client_detector::{
    compare_new_header_with_witness, detect_divergence, gather_evidence_from_conflicting_headers,
    CompareError, Error, ErrorDetail, Provider, Trace,
};
use tendermint_rpc::{Client, HttpClient, HttpClientUrl, Url};

use clap::Parser;
use color_eyre::{
    eyre::{eyre, Result},
    Report,
};
use futures::future::join_all;
use tracing::{debug, error, info, metadata::LevelFilter, warn};
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

fn parse_trust_threshold(s: &str) -> Result<TrustThreshold> {
    if let Some((l, r)) = s.split_once('/') {
        TrustThreshold::new(l.parse()?, r.parse()?).map_err(Into::into)
    } else {
        Err(eyre!(
            "invalid trust threshold: {s}, format must be X/Y where X and Y are integers"
        ))
    }
}

#[derive(Clone, Debug)]
struct List<T>(Vec<T>);

impl<E, T: FromStr<Err = E>> FromStr for List<T> {
    type Err = E;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(',')
            .map(|s| s.parse())
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
    }
}

#[derive(clap::Args, Debug, Clone)]
struct Verbosity {
    /// Increase verbosity, can be repeated up to 2 times
    #[arg(long, short, action = clap::ArgAction::Count)]
    verbose: u8,
}

impl Verbosity {
    fn to_level_filter(&self) -> LevelFilter {
        match self.verbose {
            0 => LevelFilter::INFO,
            1 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        }
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Identifier of the chain
    #[clap(long)]
    chain_id: String,

    /// Primary RPC address
    #[clap(long)]
    primary: HttpClientUrl,

    /// Comma-separated list of witnesses RPC addresses
    #[clap(long)]
    witnesses: List<HttpClientUrl>,

    /// Height of trusted header
    #[clap(long)]
    trusted_height: Height,

    /// Hash of trusted header
    #[clap(long)]
    trusted_hash: Hash,

    /// Height of the header to verify
    #[clap(long)]
    height: Option<Height>,

    /// Trust threshold
    #[clap(long, value_parser = parse_trust_threshold, default_value_t = TrustThreshold::TWO_THIRDS)]
    trust_threshold: TrustThreshold,

    /// Trusting period, in seconds (default: two weeks)
    #[clap(long, default_value = "1209600")]
    trusting_period: u64,

    /// Maximum clock drift, in seconds
    #[clap(long, default_value = "5")]
    max_clock_drift: u64,

    /// Maximum block lag, in seconds
    #[clap(long, default_value = "5")]
    max_block_lag: u64,

    /// Increase verbosity
    #[clap(flatten)]
    verbose: Verbosity,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Cli::parse();

    let env_filter = EnvFilter::builder()
        .with_default_directive(args.verbose.to_level_filter().into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(env_filter)
        .finish()
        .init();

    let options = Options {
        trust_threshold: args.trust_threshold,
        trusting_period: Duration::from_secs(args.trusting_period),
        clock_drift: Duration::from_secs(args.max_clock_drift),
    };

    let mut primary = make_provider(
        &args.chain_id,
        args.primary,
        args.trusted_height,
        args.trusted_hash,
        options,
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

    let witnesses = join_all(args.witnesses.0.into_iter().map(|addr| {
        make_provider(
            &args.chain_id,
            addr,
            trusted_block.height(),
            trusted_block.signed_header.header.hash(),
            options,
        )
    }))
    .await;

    let mut witnesses = witnesses.into_iter().collect::<Result<Vec<_>>>()?;

    let max_clock_drift = Duration::from_secs(args.max_clock_drift);
    let max_block_lag = Duration::from_secs(args.max_block_lag);
    let now = Time::now();

    run_detector(
        &mut primary,
        witnesses.as_mut_slice(),
        primary_trace,
        max_clock_drift,
        max_block_lag,
        now,
    )
    .await?;

    Ok(())
}

async fn run_detector(
    primary: &mut Provider,
    witnesses: &mut [Provider],
    primary_trace: Vec<LightBlock>,
    max_clock_drift: Duration,
    max_block_lag: Duration,
    now: Time,
) -> Result<(), Report> {
    if witnesses.is_empty() {
        return Err(Error::no_witnesses().into());
    }

    info!(
        "Running misbehavior detection against {} witnesses...",
        witnesses.len()
    );

    let primary_trace = Trace::new(primary_trace)?;

    let last_verified_block = primary_trace.last();
    let last_verified_header = &last_verified_block.signed_header;

    for witness in witnesses {
        let divergence = detect_divergence(
            Some(primary),
            witness,
            primary_trace.clone().into_vec(),
            max_clock_drift,
            max_block_lag,
        )
        .await;

        let evidence = match divergence {
            Ok(Some(divergence)) => divergence.evidence,
            Ok(None) => {
                info!(
                    "no divergence found between primary and witness {}",
                    witness.peer_id()
                );

                continue;
            },
            Err(e) => {
                error!(
                    "failed to run attack detector against witness {}: {e}",
                    witness.peer_id()
                );

                continue;
            },
        };

        // Report the evidence to the witness
        witness
            .report_evidence(Evidence::from(evidence.against_primary))
            .await
            .map_err(|e| eyre!("failed to report evidence to witness: {}", e))?;

        if let Some(against_witness) = evidence.against_witness {
            // Report the evidence to the primary
            primary
                .report_evidence(Evidence::from(against_witness))
                .await
                .map_err(|e| eyre!("failed to report evidence to primary: {}", e))?;
        }
    }

    Ok(())
}

async fn make_provider(
    chain_id: &str,
    rpc_addr: HttpClientUrl,
    trusted_height: Height,
    trusted_hash: Hash,
    options: Options,
) -> Result<Provider> {
    use tendermint_rpc::client::CompatMode;

    let rpc_client = HttpClient::builder(rpc_addr)
        .compat_mode(CompatMode::V0_34)
        .build()?;

    let node_id = rpc_client.status().await?.node_info.id;
    let light_store = Box::new(MemoryStore::new());

    let instance =
        LightClientBuilder::prod(node_id, rpc_client.clone(), light_store, options, None)
            .trust_primary_at(trusted_height, trusted_hash)?
            .build();

    Ok(Provider::new(chain_id.to_string(), instance, rpc_client))
}
