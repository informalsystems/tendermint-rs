//! Subcommands of the `tmkms` command-line application

use crate::{chain, prelude::*};
use abscissa_core::{Command, Runnable};
use std::{path::PathBuf, process};
use tendermint::amino_types::{
    vote::{SignVoteRequest, Vote},
    SignableMsg, SignedMsgType,
};

/// `ledger` subcommand
#[derive(Command, Debug, Options, Runnable)]
pub enum LedgerCommand {
    /// Initialize HRS values
    #[options(help = "initialise the height/round/step")]
    Initialise(InitCommand),
}

impl LedgerCommand {
    pub(super) fn config_path(&self) -> Option<&PathBuf> {
        match self {
            LedgerCommand::Initialise(init) => init.config.as_ref(),
        }
    }
}

/// `ledger init` subcommand
#[derive(Command, Debug, Options)]
pub struct InitCommand {
    /// config file path
    #[options(short = "c", long = "config")]
    pub config: Option<PathBuf>,

    /// block height
    #[options(short = "h", long = "height")]
    pub height: Option<i64>,

    /// block round
    #[options(short = "r", long = "round")]
    pub round: Option<i64>,
}

impl Runnable for InitCommand {
    fn run(&self) {
        let config = app_config();

        chain::load_config(&config).unwrap_or_else(|e| {
            status_err!("error loading configuration: {}", e);
            process::exit(1);
        });

        let chain_id = config.validator[0].chain_id;
        let registry = chain::REGISTRY.get();
        let chain = registry.get_chain(&chain_id).unwrap();

        let mut vote = Vote::default();
        vote.height = self.height.unwrap();
        vote.round = self.round.unwrap();
        vote.vote_type = SignedMsgType::Proposal.to_u32();
        println!("{:?}", vote);
        let sign_vote_req = SignVoteRequest { vote: Some(vote) };
        let mut to_sign = vec![];
        sign_vote_req
            .sign_bytes(config.validator[0].chain_id, &mut to_sign)
            .unwrap();

        let _sig = chain.keyring.sign_ed25519(None, &to_sign).unwrap();

        println!(
            "Successfully called the init command with height {}, and round {}",
            self.height.unwrap(),
            self.round.unwrap()
        );
    }
}
