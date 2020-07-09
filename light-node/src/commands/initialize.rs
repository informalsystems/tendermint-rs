//! `intialize` subcommand

use abscissa_core::{Command, Options, Runnable};
use crate::application::app_config;

/// `intialize` subcommand
#[derive(Command, Debug, Default, Options)]
pub struct InitCmd {
    #[options(
        free,
        help = "subjective height of the initial trusted state to initialize the node with",
    )]
    pub height: u64,

    #[options(
        free,
        help = "subjective hash of the initial validator set to initialize the node with",
    )]
    pub validators_hash: String,
}

impl Runnable for InitCmd {
    fn run(&self) {
        // TODO make an instance from primary and init store using its rpc endpoint
        // compare valset hash with subjective init one

        // gloabl conf to get primary etc
        let _cfg = app_config();
    }
}
