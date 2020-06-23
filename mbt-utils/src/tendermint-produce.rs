use gumdrop::Options;
use std::io::{self, Read};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use signatory_dalek::Ed25519Signer;
use signatory::ed25519;
use signatory::public_key::PublicKeyed;
use tendermint::*;
use validator::{Info, ProposerPriority};
use tendermint::vote::Power;
use tendermint::public_key::{PublicKey, Algorithm};
use tendermint::block::*;
use tendermint::block::header::Version;
use tendermint::{Time, validator, chain};
use tendermint::lite::ValidatorSet;
use std::str::FromStr;
use simple_error::*;


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
    #[options(help = "produce commit, from an array of validators passed via STDIN")]
    Commit(CommitOpts),
}

fn run() -> Result<(), SimpleError> {
    let opts = CliOptions::parse_args_default_or_exit();
    let res = match opts.command {
        None => {
            eprintln!("Produce tendermint datastructures from minimal input");
            eprintln!("Please specify a command:");
            eprintln!("{}\n", CliOptions::command_list().unwrap());
            eprintln!("{}\n", CliOptions::usage());
            std::process::exit(1);
        }
        Some(Command::Validator(opts)) => produce_validator(opts),
        Some(Command::Header(opts)) => produce_header(opts),
        Some(Command::Commit(opts)) => produce_commit(opts),
    }?;
    println!("{}", res);
    Ok(())
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e)
    }
}


#[derive(Debug, Options, Deserialize)]
struct ValidatorOpts {
    #[options(help = "print this help and exit")]
    #[serde(skip)]
    help: bool,
    #[options(help = "validator id (required; can be passed via STDIN)")]
    id: Option<String>,
    #[options(help = "voting power of this validator (default: 0)", meta = "POWER")]
    voting_power: Option<u64>,
    #[options(help = "proposer priority of this validator (default: none)", meta = "PRIORITY")]
    proposer_priority: Option<i64>,
}

// tries to parse STDIN as the given type; otherwise returns the raw input wrapped in SimpleError
fn read_input_as<T: DeserializeOwned>() -> Result<T, SimpleError> {
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
        Err(_) => Err(SimpleError::new("")),
        Ok(_) => {
            match serde_json::from_str(&buffer) {
                Ok(res) => Ok(res),
                Err(_) => {
                    Err(SimpleError::new(buffer))
                }
            }
        }
    }
}

fn produce_validator(cli: ValidatorOpts) -> Result<String, SimpleError> {
    let input = match read_input_as::<ValidatorOpts>() {
        Ok(input) => input,
        Err(input) => {
            ValidatorOpts {
                help: false,
                id: if input.to_string().len()==0 { None } else { Some (input.to_string()) },
                voting_power: None,
                proposer_priority: None
            }
        }
    };
    let id = choose_from_two(cli.id, input.id);
    if let None = id {
        bail!("validator identifier is missing")
    }
    let mut bytes = id.unwrap().into_bytes();
    if bytes.len() > 32 {
        bail!("identifier is too long")
    }
    bytes.extend(vec![0u8; 32 - bytes.len()].iter());
    let seed = ed25519::Seed::from_bytes(bytes).unwrap();
    let signer = Ed25519Signer::from(&seed);
    let pk = signer.public_key().unwrap();
    let info = Info {
        address: account::Id::from(pk),
        pub_key: PublicKey::from(pk),
        voting_power: Power::new(choose_from(cli.voting_power, input.voting_power, 0)),
        proposer_priority: match choose_from_two(cli.proposer_priority, input.proposer_priority) {
            Some(x) => Some(ProposerPriority::new(x)),
            None => None
        }
    };
    Ok(try_with!(serde_json::to_string(&info), "failed to serialize into JSON"))
}

#[derive(Debug, Options)]
struct HeaderOpts {
    #[options(help = "print this help and exit")]
    help: bool,
}

fn produce_header(_opts: HeaderOpts) -> Result<String, SimpleError> {
    let vals = read_input_as::<Vec<Info>>()?;
    if vals.is_empty() {
        bail!("can't produce a header for empty validator array")
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
        consensus_hash: valset.hash(), // TODO: currently not clear how to produce a valid hash
        app_hash: vec![],
        last_results_hash: None,
        evidence_hash: None,
        proposer_address: vals[0].address.clone()
    };
    Ok(try_with!(serde_json::to_string(&header), "failed to serialize into JSON"))
}


#[derive(Debug, Options, Deserialize)]
struct CommitOpts {
    #[options(help = "print this help and exit")]
    #[serde(skip)]
    help: bool,

    #[options(help = "commit round (default: 1)")]
    round: Option<u64>
}

fn produce_commit(cli: CommitOpts) -> Result<String, SimpleError> {
    const EXAMPLE_SHA256_ID: &str =
        "26C0A41F3243C6BCD7AD2DFF8A8D83A71D29D307B5326C227F734A1A512FE47D";

    let input = read_input_as::<CommitOpts>()?;
    let commit = Commit {
        height: Default::default(),
        round: choose_from(cli.round, input.round, 1),
        block_id: Id::from_str(EXAMPLE_SHA256_ID).unwrap(),
        signatures: Default::default()
    };
    Ok(try_with!(serde_json::to_string(&commit), "failed to serialize into JSON"))
}

fn choose_from<T>(cli: Option<T>, input: Option<T>, default: T) -> T {
    if let Some(x) = cli { x }
    else if let Some(y) = input { y }
    else { default }
}

fn choose_from_two<T>(cli: Option<T>, input: Option<T>) -> Option<T> {
    if let Some(x) = cli { Some(x) }
    else if let Some(y) = input { Some(y) }
    else { None }
}

// Default consensus params modeled after Go code; but it's not clear how to go to a valid hash from here
fn _default_consensus_params() -> consensus::Params {
    consensus::Params {
        block: block::Size {
            max_bytes: 22020096,
            max_gas: -1
            // Tendetmint-go also has TimeIotaMs: 1000, // 1s
        },
        evidence: evidence::Params {
            max_age_num_blocks: 100000,
            max_age_duration: evidence::Duration(std::time::Duration::new(48*3600,0))
        },
        validator: consensus::params::ValidatorParams {
            pub_key_types: vec![Algorithm::Ed25519]
        }
    }
}