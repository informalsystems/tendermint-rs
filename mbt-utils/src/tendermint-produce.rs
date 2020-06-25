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

const USAGE: &str = r#"
This is a small utility for producing tendermint datastructures
from minimal input (for testing purposes only).

For example, a tendermint validator can be produced only from an identifier,
or a tendermint header only from a set of validators.

To get an idea which input is needed for each datastructure, try '--help CMD':
it will list the required and optional parameters.

The parameters can be supplied in two ways:
  - via STDIN: in that case they are expected to be a valid JSON object,
    with each parameter being a field of this object
  - via command line arguments to the specific command.

If a parameter is supplied both via STDIN and CLI, the latter is given preference.

In case a particular datastructure can be produced from a single parameter
(like validator), there is a shortcut that allows to provide this parameter
directly via STDIN, without wrapping it into JSON object.
E.g., in the validator case, the following are equivalent:

    mbt-tendermint-produce validator --id a --voting-power 3
    echo -n '{"id": "a", "voting_power": 3}' | mbt-tendermint-produce validator
    echo -n a | mbt-tendermint-produce validator --voting-power 3
    echo -n '{"id": "a"}' | mbt-tendermint-produce validator --voting-power 3
    echo -n '{"id": "a", "voting_power": 100}' | mbt-tendermint-produce validator --voting-power 3

The result is:
    {
      "address": "730D3D6B2E9F4F0F23879458F2D02E0004F0F241",
      "pub_key": {
        "type": "tendermint/PubKeyEd25519",
        "value": "YnT69eNDaRaNU7teDTcyBedSD0B/Ziqx+sejm0wQba0="
      },
      "voting_power": "3",
      "proposer_priority": null
    }
"#;


#[derive(Debug, Options)]
struct CliOptions {
    #[options(help = "print this help and exit (--help CMD for command-specific help)")]
    help: bool,
    #[options(help = "provide detailed usage instructions")]
    usage: bool,
    #[options(help = "do not try to parse input from STDIN")]
    ignore_stdin: bool,

    #[options(command)]
    command: Option<Command>,
}

#[derive(Debug, Options)]
enum Command {
    #[options(help = "produce validator from identifier and other parameters")]
    Validator(Validator),
    #[options(help = "produce header from validator array and other parameters")]
    Header(HeaderOpts),
    #[options(help = "produce commit from validator array and other parameters")]
    Commit(CommitOpts),
}

fn print_params(options: &str) {
    for line in options.lines().skip(1) {
        eprintln!("{}", line);
    }
}

fn run_command<Opts: Producer<T> + Options, T: serde::Serialize>(cli: Opts, ignore_stdin: bool) {
    let res = if ignore_stdin { Opts::encode(&cli) }
    else { Opts::encode_with_input(&cli) };
    match res {
        Ok(res) => println!("{}", res),
        Err(e) => {
            println!("Error: {}\n", e);
            println!("Supported parameters for this command are: ");
            print_params(cli.self_usage())
        }
    }
}

fn main() {
    let opts = CliOptions::parse_args_default_or_exit();
    if opts.usage {
        eprintln!("{}", USAGE);
        std::process::exit(1);
    }
    match opts.command {
        None => {
            eprintln!("Produce tendermint datastructures for testing from minimal input\n");
            eprintln!("Please specify a command:");
            eprintln!("{}\n", CliOptions::command_list().unwrap());
            eprintln!("{}\n", CliOptions::usage());
            for cmd in CliOptions::command_list().unwrap().split("\n").map(|s| s.split_whitespace().next().unwrap()) {
                eprintln!("\n{} parameters:", cmd);
                print_params(CliOptions::command_usage(cmd).unwrap())
            }
            std::process::exit(1);
        }
        Some(Command::Validator(cli)) => run_command(cli, opts.ignore_stdin),
        Some(Command::Header(cli)) => (), //encode_header(cli),
        Some(Command::Commit(cli)) => (), //encode_commit(cli),
    }
}

trait Producer<Output: serde::Serialize> {
    fn parse_input() -> Self;
    fn combine_inputs(cli: &Self, stdin: &Self) -> Self;
    fn produce(opts: &Self) -> Result<Output, SimpleError>;
    fn encode(opts: &Self) -> Result<String, SimpleError>
        where Self: std::marker::Sized {
        let res = Self::produce(&opts)?;
        Ok(try_with!(serde_json::to_string_pretty(&res), "failed to serialize into JSON"))
    }
    fn encode_with_input(cli: &Self) -> Result<String, SimpleError>
        where Self: std::marker::Sized {
        let stdin = Self::parse_input();
        let input = Self::combine_inputs(cli, &stdin);
        let res = Self::produce(&input)?;
        Ok(try_with!(serde_json::to_string_pretty(&res), "failed to serialize into JSON"))
    }
}

impl Producer<Info> for Validator {
    fn parse_input() -> Self {
        match parse_stdin_as::<Validator>() {
            Ok(input) => input,
            Err(input) => {
                Validator {
                    id: if input.to_string().len()==0 { None } else { Some (input.to_string()) },
                    voting_power: None,
                    proposer_priority: None
                }
            }
        }
    }
    fn combine_inputs(cli: &Self, stdin: &Self) -> Self {
        Validator {
            id: choose_from_two(&cli.id, &stdin.id),
            voting_power: choose_from_two(&cli.voting_power, &stdin.voting_power),
            proposer_priority: choose_from_two(&cli.proposer_priority, &stdin.proposer_priority)
        }
    }
    fn produce(input: &Self) -> Result<Info, SimpleError> {
        if let None = input.id {
            bail!("validator identifier is missing")
        }
        let mut bytes = input.id.clone().unwrap().into_bytes();
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
            voting_power: Power::new(choose_or(input.voting_power, 0)),
            proposer_priority: match input.proposer_priority {
                None => None,
                Some(p) => Some(ProposerPriority::new(p))
            }

        };
        Ok(info)
    }
}

#[derive(Debug, Options, Deserialize, Clone)]
pub struct Validator {
    #[options(help = "validator id (required; can be passed via STDIN)")]
    id: Option<String>,
    #[options(help = "voting power of this validator (default: 0)", meta = "POWER")]
    voting_power: Option<u64>,
    #[options(help = "proposer priority of this validator (default: none)", meta = "PRIORITY")]
    proposer_priority: Option<i64>,
}

#[derive(Debug, Options, Deserialize)]
pub struct HeaderOpts {
    #[options(help = "validators (required)", parse(try_from_str = "parse_as::<Vec<Validator>>"))]
    validators: Option<Vec<Validator>>,
    #[options(help = "next validators (default: same as validators)", parse(try_from_str = "parse_as::<Vec<Validator>>"))]
    next_validators: Option<Vec<Validator>>,
    #[options(help = "block height (default: 1)",
        parse(try_from_str = "parse_as::<Height>"))]
    height: Option<Height>,
    #[options(help = "time (default: now)")]
    time: Option<Time>,
}

fn parse_header_opts(cli: HeaderOpts) -> HeaderOpts {
    // if cli.ignore_stdin {
    //     return cli
    // }
    let input = match parse_stdin_as::<HeaderOpts>() {
        Ok(input) => input,
        Err(input) => {
            HeaderOpts {
                validators: match parse_as::<Vec<Validator>>(input.as_str()) {
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
        validators: choose_from_two(&cli.validators, &input.validators),
        next_validators: choose_from_two(&cli.next_validators, &input.next_validators),
        height: choose_from_two(&cli.height, &input.height),
        time: choose_from_two(&cli.time, &input.time)
    }
}

fn produce_validators(vals: Vec<Validator>) -> Result<Vec<Info>, SimpleError> {
    Ok(vals.iter().map(|v| Validator::produce(v).unwrap()).collect())
}

pub fn produce_header(input: HeaderOpts) -> Result<Header, SimpleError> {
    if let None = input.validators {
        bail!("validator array is missing")
    }
    let vals = produce_validators(input.validators.unwrap())?;
    let valset = validator::Set::new(vals.clone());
    let next_valset = match input.next_validators {
        Some(next_vals) => validator::Set::new(produce_validators(next_vals)?.clone()),
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
pub struct CommitOpts {
    #[options(help = "block height (default: 1)",
    parse(try_from_str = "parse_as::<Height>"))]
    height: Option<Height>,
    #[options(help = "commit round (default: 1)")]
    round: Option<u64>
}

fn parse_commit_opts(cli: CommitOpts) -> CommitOpts {
    // if cli.ignore_stdin {
    //     return cli
    // }
    let input = match parse_stdin_as::<CommitOpts>() {
        Ok(input) => input,
        Err(_) => CommitOpts {
            height: None,
            round: None
        }
    };
    CommitOpts {
        height: choose_from_two(&cli.height, &input.height),
        round: choose_from_two(&cli.round, &input.round)
    }
}


pub fn produce_commit(input: CommitOpts) -> Result<Commit, SimpleError> {
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


// fn _choose_from<T>(cli: &Option<T>, input: &Option<T>, default: T) -> T {
//     if let Some(x) = cli { x }
//     else if let Some(y) = input { y }
//     else { default }
// }

fn choose_from_two<T: Clone>(cli: &Option<T>, input: &Option<T>) -> Option<T> {
    if let Some(x) = cli { Some(x.clone()) }
    else if let Some(y) = input { Some(y.clone()) }
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