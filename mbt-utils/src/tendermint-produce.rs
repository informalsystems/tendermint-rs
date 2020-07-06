use gumdrop::Options;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use signatory::ed25519;
use signatory::ed25519::SIGNATURE_SIZE;
use signatory::public_key::PublicKeyed;
use signatory::signature::Signature as _;
use signatory::signature::Signer;
use signatory_dalek::Ed25519Signer;
use simple_error::*;
use std::io::{self, Read};
use std::str::FromStr;
use subtle_encoding::base64;
use subtle_encoding::hex::encode;
use tendermint::amino_types::message::AminoMessage;
use tendermint::block::header::Version;
use tendermint::lite::ValidatorSet;
use tendermint::private_key::Ed25519Keypair;
use tendermint::public_key::{Algorithm, PublicKey};
use tendermint::vote::{Power, SignedVote, Type};
use tendermint::*;
use tendermint::{chain, validator, Time};
use tendermint_light_client::operations::{Hasher, ProdHasher};
use validator::{Info, ProposerPriority};

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
    echo -n '{"id": "a", "voting_power": 3}' | mbt-tendermint-produce --read-stdin validator
    echo -n a | mbt-tendermint-produce --read-stdin validator --voting-power 3
    echo -n '{"id": "a"}' | mbt-tendermint-produce --read-stdin validator --voting-power 3
    echo -n '{"id": "a", "voting_power": 100}' | mbt-tendermint-produce --read-stdin validator --voting-power 3

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
    #[options(help = "read input from STDIN (default: no)")]
    read_stdin: bool,

    #[options(command)]
    command: Option<Command>,
}

#[derive(Debug, Options)]
enum Command {
    #[options(help = "produce validator from identifier and other parameters")]
    Validator(Validator),
    #[options(help = "produce header from validator array and other parameters")]
    Header(Header),
    #[options(help = "produce commit from validator array and other parameters")]
    Commit(Commit),
}

fn run_command<Opts: Producer<T> + Options, T: serde::Serialize>(cli: Opts, read_stdin: bool) {
    let res = if read_stdin {
        Opts::encode_with_stdin(&cli)
    } else {
        Opts::encode(&cli)
    };
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
            for cmd in CliOptions::command_list()
                .unwrap()
                .split("\n")
                .map(|s| s.split_whitespace().next().unwrap())
            {
                eprintln!("\n{} parameters:", cmd);
                print_params(CliOptions::command_usage(cmd).unwrap())
            }
            std::process::exit(1);
        }
        Some(Command::Validator(cli)) => run_command(cli, opts.read_stdin),
        Some(Command::Header(cli)) => run_command(cli, opts.read_stdin),
        Some(Command::Commit(cli)) => run_command(cli, opts.read_stdin),
    }
}

pub trait Producer<Output: serde::Serialize> {
    fn parse_stdin() -> Result<Self, SimpleError>
    where
        Self: std::marker::Sized;
    fn merge_with_default(&self, other: &Self) -> Self;
    fn produce(&self) -> Result<Output, SimpleError>;
    fn encode(&self) -> Result<String, SimpleError>
    where
        Self: std::marker::Sized,
    {
        let res = self.produce()?;
        Ok(try_with!(
            serde_json::to_string_pretty(&res),
            "failed to serialize into JSON"
        ))
    }
    fn encode_with_stdin(&self) -> Result<String, SimpleError>
    where
        Self: std::marker::Sized,
    {
        let stdin = Self::parse_stdin()?;
        let producer = self.merge_with_default(&stdin);
        producer.encode()
    }
}

#[derive(Debug, Options, Deserialize, Clone)]
pub struct Validator {
    #[options(help = "validator id (required; can be passed via STDIN)")]
    id: Option<String>,
    #[options(help = "voting power of this validator (default: 0)", meta = "POWER")]
    voting_power: Option<u64>,
    #[options(
        help = "proposer priority of this validator (default: none)",
        meta = "PRIORITY"
    )]
    proposer_priority: Option<i64>,
}

impl Validator {
    pub fn new(id: &str) -> Self {
        Validator {
            id: Some(id.to_string()),
            voting_power: None,
            proposer_priority: None,
        }
    }
    pub fn voting_power(mut self, power: u64) -> Self {
        self.voting_power = Some(power);
        self
    }
    pub fn proposer_priority(mut self, priority: i64) -> Self {
        self.proposer_priority = Some(priority);
        self
    }
    pub fn signer(&self) -> Result<Ed25519Signer, SimpleError> {
        if let None = self.id {
            bail!("validator identifier is missing")
        }
        let mut bytes = self.id.clone().unwrap().into_bytes();
        if bytes.len() > 32 {
            bail!("identifier is too long")
        }
        bytes.extend(vec![0u8; 32 - bytes.len()].iter());
        let seed = require_with!(
            ed25519::Seed::from_bytes(bytes),
            "failed to construct a seed"
        );
        Ok(Ed25519Signer::from(&seed))
    }
}

impl Producer<Info> for Validator {
    fn parse_stdin() -> Result<Self, SimpleError> {
        let validator = match parse_stdin_as::<Validator>() {
            Ok(input) => input,
            Err(input) => Validator {
                id: if input.to_string().len() == 0 {
                    bail!("failed to read validator from input")
                } else {
                    Some(input.to_string())
                },
                voting_power: None,
                proposer_priority: None,
            },
        };
        Ok(validator)
    }
    fn merge_with_default(&self, other: &Self) -> Self {
        Validator {
            id: choose_from(&self.id, &other.id),
            voting_power: choose_from(&self.voting_power, &other.voting_power),
            proposer_priority: choose_from(&self.proposer_priority, &other.proposer_priority),
        }
    }
    fn produce(&self) -> Result<Info, SimpleError> {
        let signer = self.signer()?;
        let pk = try_with!(signer.public_key(), "failed to get a public key");
        let info = Info {
            address: account::Id::from(pk),
            pub_key: PublicKey::from(pk),
            voting_power: Power::new(choose_or(self.voting_power, 0)),
            proposer_priority: match self.proposer_priority {
                None => None,
                Some(p) => Some(ProposerPriority::new(p)),
            },
        };
        Ok(info)
    }
}

#[derive(Debug, Options, Deserialize, Clone)]
pub struct Header {
    #[options(
        help = "validators (required), encoded as array of 'validator' parameters",
        parse(try_from_str = "parse_as::<Vec<Validator>>")
    )]
    validators: Option<Vec<Validator>>,
    #[options(
        help = "next validators (default: same as validators), encoded as array of 'validator' parameters",
        parse(try_from_str = "parse_as::<Vec<Validator>>")
    )]
    next_validators: Option<Vec<Validator>>,
    #[options(help = "block height (default: 1)")]
    height: Option<u64>,
    #[options(help = "time (default: now)")]
    time: Option<Time>,
}

impl Header {
    pub fn new(validators: &Vec<Validator>) -> Self {
        Header {
            validators: Some(validators.clone()),
            next_validators: None,
            height: None,
            time: None,
        }
    }
    pub fn next_validators(mut self, vals: &Vec<Validator>) -> Self {
        self.next_validators = Some(vals.clone());
        self
    }
    pub fn height(mut self, height: u64) -> Self {
        self.height = Some(height);
        self
    }
    pub fn time(mut self, time: Time) -> Self {
        self.time = Some(time);
        self
    }
}

impl Producer<block::Header> for Header {
    fn parse_stdin() -> Result<Self, SimpleError> {
        let header = match parse_stdin_as::<Header>() {
            Ok(input) => input,
            Err(input) => Header {
                validators: match parse_as::<Vec<Validator>>(input.as_str()) {
                    Ok(vals) => Some(vals),
                    Err(e) => bail!("failed to read header from input"),
                },
                next_validators: None,
                height: None,
                time: None,
            },
        };
        Ok(header)
    }

    fn merge_with_default(&self, other: &Self) -> Self {
        Header {
            validators: choose_from(&self.validators, &other.validators),
            next_validators: choose_from(&self.next_validators, &other.next_validators),
            height: choose_from(&self.height, &other.height),
            time: choose_from(&self.time, &other.time),
        }
    }

    fn produce(&self) -> Result<block::Header, SimpleError> {
        if let None = self.validators {
            bail!("validator array is missing")
        }
        let vals = produce_validators(&self.validators.as_ref().unwrap())?;
        let valset = validator::Set::new(vals.clone());
        let next_valset = match &self.next_validators {
            Some(next_vals) => validator::Set::new(produce_validators(next_vals)?.clone()),
            None => valset.clone(),
        };
        let header = block::Header {
            version: Version { block: 0, app: 0 },
            chain_id: chain::Id::from_str("test-chain-01").unwrap(),
            height: block::Height(choose_or(self.height, 1)),
            time: choose_or(self.time, Time::now()),
            last_block_id: None,
            last_commit_hash: None,
            data_hash: None,
            validators_hash: valset.hash(),
            next_validators_hash: next_valset.hash(), // hasher.hash_validator_set(&next_valset), // next_valset.hash(),
            consensus_hash: valset.hash(), //hasher.hash_validator_set(&valset), // TODO: currently not clear how to produce a valid hash
            app_hash: vec![],
            last_results_hash: None,
            evidence_hash: None,
            proposer_address: vals[0].address.clone(),
        };
        Ok(header)
    }
}

fn produce_validators(vals: &Vec<Validator>) -> Result<Vec<Info>, SimpleError> {
    Ok(vals.iter().map(|v| v.produce().unwrap()).collect())
}

#[derive(Debug, Options, Deserialize)]
pub struct Commit {
    #[options(help = "header (required)", parse(try_from_str = "parse_as::<Header>"))]
    header: Option<Header>,
    #[options(help = "commit round (default: 1)")]
    round: Option<u64>,
}

impl Commit {
    pub fn new(header: &Header) -> Self {
        Commit {
            header: Some(header.clone()),
            round: None,
        }
    }
    pub fn round(mut self, round: u64) -> Self {
        self.round = Some(round);
        self
    }
}

impl Producer<block::Commit> for Commit {
    fn parse_stdin() -> Result<Self, SimpleError> {
        let commit = match parse_stdin_as::<Commit>() {
            Ok(input) => input,
            Err(input) => Commit {
                header: match parse_as::<Header>(input.as_str()) {
                    Ok(header) => Some(header),
                    Err(e) => bail!("failed to read commit from input"),
                },
                round: None,
            },
        };
        Ok(commit)
    }

    fn merge_with_default(&self, other: &Self) -> Self {
        Commit {
            header: choose_from(&self.header, &other.header),
            round: choose_from(&self.round, &other.round),
        }
    }

    fn produce(&self) -> Result<block::Commit, SimpleError> {
        if let None = self.header {
            bail!("header is missing")
        }
        let header = self.header.as_ref().unwrap();
        let block_header = header.produce()?;
        let hasher = ProdHasher;
        let block_id = block::Id::new(lite::Header::hash(&block_header), None);
        let sigs: Vec<block::CommitSig> = header
            .validators
            .as_ref()
            .unwrap()
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                let validator = v.produce().unwrap();
                let signer: Ed25519Signer = v.signer().unwrap();
                let vote = Vote {
                    vote_type: Type::Precommit,
                    height: block_header.height,
                    round: choose_or(self.round, 1),
                    block_id: Some(block_id.clone()),
                    timestamp: block_header.time,
                    validator_address: validator.address,
                    validator_index: i as u64,
                    signature: Signature::Ed25519(
                        ed25519::Signature::from_bytes(&vec![0_u8; SIGNATURE_SIZE]).unwrap(),
                    ),
                };
                let signed_vote = vote::SignedVote::new(
                    amino_types::vote::Vote::from(&vote),
                    block_header.chain_id.as_str(),
                    validator.address,
                    Signature::Ed25519(
                        ed25519::Signature::from_bytes(&vec![0_u8; SIGNATURE_SIZE]).unwrap(),
                    ),
                );
                let sign_bytes = signed_vote.sign_bytes();
                let sig = block::CommitSig::BlockIDFlagCommit {
                    validator_address: validator.address,
                    timestamp: block_header.time,
                    signature: Signature::Ed25519(signer.try_sign(sign_bytes.as_slice()).unwrap()),
                };
                sig
            })
            .collect();

        let commit = block::Commit {
            height: block_header.height,
            round: choose_or(self.round, 1),
            block_id: block_id, // TODO do we need at least one part? //block::Id::new(hasher.hash_header(&block_header), None), //
            signatures: block::CommitSigs::new(sigs),
        };
        Ok(commit)
    }
}

// Helper functions

fn print_params(options: &str) {
    for line in options.lines().skip(1) {
        eprintln!("{}", line);
    }
}

// tries to parse a string as the given type; otherwise returns the input wrapped in SimpleError
fn parse_as<T: DeserializeOwned>(input: &str) -> Result<T, SimpleError> {
    match serde_json::from_str(input) {
        Ok(res) => Ok(res),
        Err(_) => Err(SimpleError::new(input)),
    }
}

// tries to parse STDIN as the given type; otherwise returns the input wrapped in SimpleError
fn parse_stdin_as<T: DeserializeOwned>() -> Result<T, SimpleError> {
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
        Err(_) => Err(SimpleError::new("")),
        Ok(_) => parse_as::<T>(&buffer),
    }
}

fn choose_or<T>(input: Option<T>, default: T) -> T {
    if let Some(x) = input {
        x
    } else {
        default
    }
}

fn choose_from<T: Clone>(cli: &Option<T>, input: &Option<T>) -> Option<T> {
    if let Some(x) = cli {
        Some(x.clone())
    } else if let Some(y) = input {
        Some(y.clone())
    } else {
        None
    }
}

// Default consensus params modeled after Go code; but it's not clear how to go to a valid hash from here
fn _default_consensus_params() -> consensus::Params {
    consensus::Params {
        block: block::Size {
            max_bytes: 22020096,
            max_gas: -1, // Tendetmint-go also has TimeIotaMs: 1000, // 1s
        },
        evidence: evidence::Params {
            max_age_num_blocks: 100000,
            max_age_duration: evidence::Duration(std::time::Duration::new(48 * 3600, 0)),
        },
        validator: consensus::params::ValidatorParams {
            pub_key_types: vec![Algorithm::Ed25519],
        },
    }
}

