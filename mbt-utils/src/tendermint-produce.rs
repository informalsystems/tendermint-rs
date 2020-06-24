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
    #[options(help = "produce validator from identifier")]
    Validator(ValidatorOpts),
    #[options(help = "produce header from validator array")]
    Header(HeaderOpts),
    #[options(help = "produce commit from validator array")]
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
        Some(Command::Validator(opts)) => encode_validator(opts),
        Some(Command::Header(opts)) => encode_header(opts),
        Some(Command::Commit(opts)) => encode_commit(opts),
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
pub struct ValidatorOpts {
    #[options(help = "print this help and exit")]
    #[serde(skip)]
    help: bool,
    #[options(help = "do not try to parse input from STDIN")]
    #[serde(skip)]
    ignore_stdin: bool,
    #[options(help = "validator id (required; can be passed via STDIN)")]
    id: Option<String>,
    #[options(help = "voting power of this validator (default: 0)", meta = "POWER",
        parse(try_from_str = "parse_as::<Power>"))]
    voting_power: Option<Power>,
    #[options(help = "proposer priority of this validator (default: none)", meta = "PRIORITY",
        parse(try_from_str = "parse_as::<ProposerPriority>"))]
    proposer_priority: Option<ProposerPriority>,
}

fn parse_validator_opts(cli: ValidatorOpts) -> ValidatorOpts {
    if cli.ignore_stdin {
        return cli
    }
    let input = match parse_stdin_as::<ValidatorOpts>() {
        Ok(input) => input,
        Err(input) => {
            ValidatorOpts {
                help: false,
                ignore_stdin: false,
                id: if input.to_string().len()==0 { None } else { Some (input.to_string()) },
                voting_power: None,
                proposer_priority: None
            }
        }
    };
    ValidatorOpts {
        help: false,
        ignore_stdin: false,
        id: choose_from_two(cli.id, input.id),
        voting_power: choose_from_two(cli.voting_power, input.voting_power),
        proposer_priority: choose_from_two(cli.proposer_priority, input.proposer_priority)
    }
}

pub fn produce_validator(input: ValidatorOpts) -> Result<Info, SimpleError> {
    if let None = input.id {
        bail!("validator identifier is missing")
    }
    let mut bytes = input.id.unwrap().into_bytes();
    if bytes.len() > 32 {
        bail!("identifier is too long")
    }
    bytes.extend(vec![0u8; 32 - bytes.len()].iter());
    let seed = require_with!(ed25519::Seed::from_bytes(bytes), "failed to construct a seed");
    let signer = Ed25519Signer::from(&seed);
    let pk = try_with!(signer.public_key(), "failed to get a public key");
    let info = Info {
        address: account::Id::from(pk),
        pub_key: PublicKey::from(pk),
        voting_power: choose_or(input.voting_power, Power::new(0)),
        proposer_priority: input.proposer_priority
    };
    Ok(info)
}

fn encode_validator(cli: ValidatorOpts) -> Result<String, SimpleError> {
    let input = parse_validator_opts(cli);
    let info = produce_validator(input)?;
    Ok(try_with!(serde_json::to_string(&info), "failed to serialize validator into JSON"))
}

#[derive(Debug, Options, Deserialize)]
pub struct HeaderOpts {
    #[options(help = "print this help and exit")]
    help: bool,
    #[options(help = "do not try to parse input from STDIN")]
    #[serde(skip)]
    ignore_stdin: bool,
    #[options(help = "validators (required)", parse(try_from_str = "parse_as::<Vec<Info>>"))]
    validators: Option<Vec<Info>>,
    #[options(help = "next validators (default: same as validators)", parse(try_from_str = "parse_as::<Vec<Info>>"))]
    next_validators: Option<Vec<Info>>,
    #[options(help = "block height (default: 1)",
        parse(try_from_str = "parse_as::<Height>"))]
    height: Option<Height>,
    #[options(help = "time (default: now)")]
    time: Option<Time>,
}

fn parse_header_opts(cli: HeaderOpts) -> HeaderOpts {
    if cli.ignore_stdin {
        return cli
    }
    let input = match parse_stdin_as::<HeaderOpts>() {
        Ok(input) => input,
        Err(input) => {
            HeaderOpts {
                help: false,
                ignore_stdin: false,
                validators: match parse_as::<Vec<Info>>(input.as_str()) {
                    Ok(vals) => Some(vals),
                    Err(_) => None
                },
                next_validators: None,
                height: None,
                time: None
            }
        }
    };
    HeaderOpts {
        help: false,
        ignore_stdin: false,
        validators: choose_from_two(cli.validators, input.validators),
        next_validators: choose_from_two(cli.next_validators, input.next_validators),
        height: choose_from_two(cli.height, input.height),
        time: choose_from_two(cli.time, input.time)
    }
}

pub fn produce_header(input: HeaderOpts) -> Result<Header, SimpleError> {
    if let None = input.validators {
        bail!("validator array is missing")
    }
    let vals = input.validators.unwrap();
    let valset = validator::Set::new(vals.clone());
    let next_valset = match input.next_validators {
        Some(next_vals) => validator::Set::new(next_vals.clone()),
        None => valset.clone()
    };
    let header = Header {
        version: Version { block: 0, app: 0 },
        chain_id: chain::Id::from_str("test-chain-01").unwrap(),
        height: choose_or(input.height,Height(1)),
        time: choose_or(input.time,Time::now()),
        last_block_id: None,
        last_commit_hash: None,
        data_hash: None,
        validators_hash: valset.hash(),
        next_validators_hash: next_valset.hash(),
        consensus_hash: valset.hash(), // TODO: currently not clear how to produce a valid hash
        app_hash: vec![],
        last_results_hash: None,
        evidence_hash: None,
        proposer_address: vals[0].address.clone()
    };
    Ok(header)
}

fn encode_header(cli: HeaderOpts) -> Result<String, SimpleError> {
    let input = parse_header_opts(cli);
    let header = produce_header(input)?;
    Ok(try_with!(serde_json::to_string(&header), "failed to serialize header into JSON"))
}

#[derive(Debug, Options, Deserialize)]
struct CommitOpts {
    #[options(help = "print this help and exit")]
    #[serde(skip)]
    help: bool,
    #[options(help = "do not try to parse input from STDIN")]
    #[serde(skip)]
    ignore_stdin: bool,
    #[options(help = "block height (default: 1)",
    parse(try_from_str = "parse_as::<Height>"))]
    height: Option<Height>,
    #[options(help = "commit round (default: 1)")]
    round: Option<u64>
}

fn parse_commit_opts(cli: CommitOpts) -> CommitOpts {
    if cli.ignore_stdin {
        return cli
    }
    let input = match parse_stdin_as::<CommitOpts>() {
        Ok(input) => input,
        Err(_) => CommitOpts {
            help: false,
            ignore_stdin: false,
            height: None,
            round: None
        }
    };
    CommitOpts {
        help: false,
        ignore_stdin: false,
        height: choose_from_two(cli.height, input.height),
        round: choose_from_two(cli.round, input.round)
    }
}


fn produce_commit(input: CommitOpts) -> Result<Commit, SimpleError> {
    const EXAMPLE_SHA256_ID: &str =
        "26C0A41F3243C6BCD7AD2DFF8A8D83A71D29D307B5326C227F734A1A512FE47D";
    let commit = Commit {
        height: choose_or(input.height, Height(1)),
        round: choose_or(input.round, 1),
        block_id: Id::from_str(EXAMPLE_SHA256_ID).unwrap(),
        signatures: Default::default()
    };
    Ok(commit)
}

fn encode_commit(cli: CommitOpts) -> Result<String, SimpleError> {
    let input = parse_commit_opts(cli);
    let commit = produce_commit(input)?;
    Ok(try_with!(serde_json::to_string(&commit), "failed to serialize commit into JSON"))
}

// Helper functions

// tries to parse a string as the given type; otherwise returns the input wrapped in SimpleError
fn parse_as<T: DeserializeOwned>(input: &str) -> Result<T, SimpleError> {
    match serde_json::from_str(input) {
        Ok(res) => Ok(res),
        Err(_) => Err(SimpleError::new(input))
    }
}

// tries to parse STDIN as the given type; otherwise returns the input wrapped in SimpleError
fn parse_stdin_as<T: DeserializeOwned>() -> Result<T, SimpleError> {
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
        Err(_) => Err(SimpleError::new("")),
        Ok(_) => parse_as::<T>(&buffer)
    }
}

fn choose_or<T>(input: Option<T>, default: T) -> T {
    if let Some(x) = input { x }
    else { default }
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