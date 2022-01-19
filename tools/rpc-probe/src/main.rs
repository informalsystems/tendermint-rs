mod client;
mod common;
mod error;
mod kvstore;
mod plan;
mod request;
mod subscription;
mod utils;

use crate::kvstore::quick_probe_plan;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use tokio::time::Duration;

// Set default value of `--output` to rpc crate test folder
#[derive(Debug)]
struct OutputPathBuf(pub PathBuf);
impl Default for OutputPathBuf {
    fn default() -> Self {
        Self(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("rpc")
                .join("tests")
                .join("kvstore_fixtures"),
        )
    }
}
impl FromStr for OutputPathBuf {
    type Err = structopt::clap::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(PathBuf::from(s)))
    }
}
impl ToString for OutputPathBuf {
    fn to_string(&self) -> String {
        self.0.to_str().unwrap_or("").to_string()
    }
}

/// A utility application that primarily aims to assist in testing
/// compatibility between tendermint.rs (https://github.com/informalsystems/tendermint-rs)
/// and Tendermint (https://github.com/tendermint/tendermint).
///
/// Running this application will execute a "quick probe" against a running
/// Tendermint node. This executes a number of RPC requests against the node,
/// saving both the requests and responses to the desired output folder.
#[derive(Debug, StructOpt)]
struct Opts {
    /// The address of the Tendermint node's WebSocket-based RPC endpoint.
    #[structopt(default_value = "ws://127.0.0.1:26657/websocket", long)]
    pub addr: String,

    /// The output path in which to store the received responses.
    #[structopt(default_value, short, long)]
    pub output: OutputPathBuf,

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

    quick_probe_plan(&opts.output.0, Duration::from_millis(opts.request_wait))?
        .execute(&opts.addr)
        .await?;
    Ok(())
}
