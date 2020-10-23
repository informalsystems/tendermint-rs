mod error;
mod messages;
mod opts;
mod quick;
mod subscription;
mod websocket;

use crate::opts::GeneralOptions;
use crate::quick::quick_probe;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opts {
    #[structopt(flatten)]
    general: GeneralOptions,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Perform a general-purpose quick probe of a Tendermint node.
    ///
    /// This assumes that the Tendermint node in question is running the
    /// kvstore application.
    ///
    /// This will automatically try to execute every kind of request against
    /// the given node, recording all of its responses. For a full list of
    /// supported endpoints, please see https://docs.tendermint.com/master/rpc/
    Quick,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::from_args();
    match opts.cmd {
        Command::Quick => quick_probe(opts.general).await.map_err(Into::into),
    }
}
