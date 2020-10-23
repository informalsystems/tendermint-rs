//! Options relating to probe execution.

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct GeneralOptions {
    /// The address of the Tendermint node's WebSocket-based RPC endpoint.
    #[structopt(default_value = "ws://127.0.0.1:26657/websocket", long)]
    pub addr: String,

    /// The output path in which to store the received responses.
    #[structopt(parse(from_os_str), short, long)]
    pub output: PathBuf,

    /// Save responses within their original JSON-RPC wrappers.
    #[structopt(long)]
    pub save_wrapper: bool,

    /// Increase output logging verbosity.
    #[structopt(short, long)]
    pub verbose: bool,
}
