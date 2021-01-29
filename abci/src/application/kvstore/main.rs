//! In-memory key/value store application for Tendermint.

use log::LevelFilter;
use simple_logger::SimpleLogger;
use structopt::StructOpt;
use tendermint_abci::{KeyValueStoreApp, ServerBuilder};

#[derive(Debug, StructOpt)]
struct Opt {
    /// Bind the TCP server to this host.
    #[structopt(short, long, default_value = "127.0.0.1")]
    host: String,

    /// Bind the TCP server to this port.
    #[structopt(short, long, default_value = "26658")]
    port: u16,

    /// Increase output logging verbosity to DEBUG level.
    #[structopt(short, long)]
    verbose: bool,

    /// Suppress all output logging (overrides --verbose).
    #[structopt(short, long)]
    quiet: bool,
}

fn main() {
    let opt: Opt = Opt::from_args();
    SimpleLogger::new()
        .with_level(if opt.quiet {
            LevelFilter::Off
        } else if opt.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .init()
        .unwrap();

    let (app, driver) = KeyValueStoreApp::new();
    let server = ServerBuilder::default()
        .bind(format!("{}:{}", opt.host, opt.port), app)
        .unwrap();
    std::thread::spawn(move || driver.run());
    server.listen().unwrap();
}
