mod error;
mod plan;
mod quick;
mod request;
mod subscription;
mod utils;
mod websocket;

use crate::quick::quick_probe_plan;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::time::Duration;

#[derive(Debug, StructOpt)]
/// A utility application that primarily aims to assist in testing
/// compatibility between tendermint.rs (https://github.com/informalsystems/tendermint-rs)
/// and Tendermint (https://github.com/tendermint/tendermint).
///
/// Running this application will execute a "quick probe" against a running
/// Tendermint node. This executes a number of RPC requests against the node,
/// saving both the requests and responses to the desired output folder.
struct Opts {
    /// The address of the Tendermint node's WebSocket-based RPC endpoint.
    #[structopt(default_value = "ws://127.0.0.1:26657/websocket", long)]
    pub addr: String,

    /// The output path in which to store the received responses.
    #[structopt(default_value = "probe-results", parse(from_os_str), short, long)]
    pub output: PathBuf,

    /// How long to wait between requests, in milliseconds.
    #[structopt(default_value = "1000", long)]
    pub request_wait: u64,

    /// Increase output logging verbosity.
    #[structopt(short, long)]
    pub verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::from_args();
    let log_level = if opts.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    SimpleLogger::new().with_level(log_level).init().unwrap();

    let (plan, driver) = quick_probe_plan(&opts.output, Duration::from_millis(opts.request_wait))?
        .connect(&opts.addr)
        .await?;
    let driver_handle = tokio::spawn(async move { driver.run().await });
    plan.execute_and_close().await?;
    driver_handle.await??;
    Ok(())
}
