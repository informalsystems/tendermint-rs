use gumdrop::Options;
use std::io::{self, Read};

use signatory_dalek::Ed25519Signer;
use signatory::ed25519;
use signatory::public_key::PublicKeyed;
use tendermint::validator::{Info, ProposerPriority};
use tendermint::vote::Power;
use tendermint::public_key::PublicKey;
use std::process::exit;
use tendermint::block::*;
use tendermint::block::header::Version;
use tendermint::{Time, validator, chain};
use tendermint::lite::ValidatorSet;
use std::str::FromStr;


#[derive(Debug, Options)]
struct CliOptions {
    #[options(help = "print this help and exit")]
    help: bool,

    #[options(command)]
    command: Option<Command>,
}

#[derive(Debug, Options)]
enum Command {
    #[options(help = "produce validator from an identifier, passed via STDIN")]
    Validator(ValidatorOpts),
    #[options(help = "produce header, from an array of validators passed via STDIN")]
    Header(HeaderOpts),

}



fn main() {
    let opts = CliOptions::parse_args_default_or_exit();
    match opts.command {
        None => {
            eprintln!("Produce tendermint datastructures from minimal input");
            eprintln!("Please specify a command:");
            eprintln!("{}\n", CliOptions::command_list().unwrap());
            eprintln!("{}\n", CliOptions::usage());
            std::process::exit(1);
        }
        Some(Command::Validator(opts)) => produce_validator(opts),
        Some(Command::Header(opts)) => produce_header(opts),
    }
}


#[derive(Debug, Options)]
struct ValidatorOpts {
    #[options(help = "print this help and exit")]
    help: bool,
    #[options(help = "voting power of this validator (default: 0)", meta = "POWER")]
    voting_power: Option<u64>,
    #[options(help = "proposer priority of this validator (default: none)", meta = "PRIORITY")]
    proposer_priority: Option<i64>,
}

fn produce_validator(opts: ValidatorOpts) {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    let mut bytes = buffer.into_bytes();
    if bytes.len() > 32 {
        eprintln!("identifier is too long");
        exit(1)
    }
    bytes.extend(vec![0u8; 32 - bytes.len()].iter());
    let seed = ed25519::Seed::from_bytes(bytes).unwrap();
    let signer = Ed25519Signer::from(&seed);
    let pk = signer.public_key().unwrap();
    let mut info = Info::new(PublicKey::from(pk), Power::new(0));
    if let Some(power) = opts.voting_power {
        info.voting_power = Power::new(power);
    }
    if let Some(priority) = opts.proposer_priority {
        info.proposer_priority = Some(ProposerPriority::new(priority));
    }
    println!("{}", serde_json::to_string(&info).unwrap());
}

#[derive(Debug, Options)]
struct HeaderOpts {
    #[options(help = "print this help and exit")]
    help: bool,
}

fn produce_header(_opts: HeaderOpts) {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    if let Ok(vals) = serde_json::from_str::<Vec<Info>>(buffer.as_str()) {
        if vals.is_empty() {
            eprintln!("can't produce a header for empty validator set");
            exit(1)
        }
        let valset = validator::Set::new(vals.clone());
        let header = Header {
            version: Version { block: 0, app: 0 },
            chain_id: chain::Id::from_str("test-chain-01").unwrap(),
            height: Default::default(),
            time: Time::now(),
            last_block_id: None,
            last_commit_hash: None,
            data_hash: None,
            validators_hash: valset.hash(),
            next_validators_hash: valset.hash(),
            consensus_hash: valset.hash(),
            app_hash: vec![],
            last_results_hash: None,
            evidence_hash: None,
            proposer_address: vals[0].address.clone()
        };
        println!("{}", serde_json::to_string(&header).unwrap());
    }
}