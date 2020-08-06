use anomaly::fail;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use tendermint::block::Header;
use tendermint::lite::error::{Error, Kind};
use tendermint::lite::Requester;
use tendermint::{
    block::signed_header::SignedHeader, evidence::Duration, lite, validator::Set, Time,
};
use std::{process, io};
use std::io::Read;

/// Test that a struct `T` can be:
///
/// - serialized to JSON
/// - parsed back from the serialized JSON of the previous step
/// - that the two parsed structs are equal according to their `PartialEq` impl
pub fn test_serialization_roundtrip<T>(obj: &T)
where
    T: Debug + PartialEq + Serialize + DeserializeOwned,
{
    let serialized = serde_json::to_string(obj).unwrap();
    let parsed = serde_json::from_str(&serialized).unwrap();
    assert_eq!(obj, &parsed);
}

#[derive(Deserialize, Clone, Debug)]
pub struct Initial {
    pub signed_header: SignedHeader,
    pub next_validator_set: Set,
    pub trusting_period: Duration,
    pub now: Time,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteBlock {
    pub signed_header: SignedHeader,
    pub validator_set: Set,
    pub next_validator_set: Set,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MockRequester {
    chain_id: String,
    signed_headers: HashMap<u64, SignedHeader>,
    validators: HashMap<u64, Set>,
}

type LightSignedHeader = lite::types::SignedHeader<SignedHeader, Header>;

#[async_trait]
impl Requester<SignedHeader, Header> for MockRequester {
    async fn signed_header(&self, h: u64) -> Result<LightSignedHeader, Error> {
        println!("requested signed header for height:{:?}", h);
        if let Some(sh) = self.signed_headers.get(&h) {
            return Ok(sh.into());
        }
        println!("couldn't get sh for: {}", &h);
        fail!(Kind::RequestFailed, "couldn't get sh for: {}", &h);
    }

    async fn validator_set(&self, h: u64) -> Result<Set, Error> {
        println!("requested validators for height:{:?}", h);
        if let Some(vs) = self.validators.get(&h) {
            return Ok(vs.to_owned());
        }
        println!("couldn't get vals for: {}", &h);
        fail!(Kind::RequestFailed, "couldn't get vals for: {}", &h);
    }
}

impl MockRequester {
    pub fn new(chain_id: String, lite_blocks: Vec<LiteBlock>) -> Self {
        let mut sh_map: HashMap<u64, SignedHeader> = HashMap::new();
        let mut val_map: HashMap<u64, Set> = HashMap::new();
        let last_block = lite_blocks.last().expect("last entry not found");
        val_map.insert(
            last_block.signed_header.header.height.increment().value(),
            last_block.to_owned().next_validator_set,
        );
        for lite_block in lite_blocks {
            let height = lite_block.signed_header.header.height;
            sh_map.insert(height.into(), lite_block.signed_header);
            val_map.insert(height.into(), lite_block.validator_set);
        }
        Self {
            chain_id,
            signed_headers: sh_map,
            validators: val_map,
        }
    }
}

/// A thin wrapper around process::Command to facilitate running external commands.
pub struct Command {
    program: Option<String>,
    args: Vec<String>,
    dir: Option<String>
}

impl Command {
    /// Constructs a new Command for the given program with arguments.
    pub fn new() -> Command {
        Command {
            program: None,
            args: vec![],
            dir: None
        }
    }

    /// Sets the program to run
    pub fn program(&mut self, program: &str) -> &mut Self {
        self.program = Some(program.to_owned());
        self
    }

    /// Adds a new program argument
    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.args.push(arg.to_owned());
        self
    }

    /// Adds a new program argument, concatenated from several parts
    pub fn arg_from_parts(&mut self, parts: Vec<&str>) -> &mut Self {
        let mut arg: String = String::new();
        for part in parts {
            arg = arg + part;
        }
        self.args.push(arg);
        self
    }

    /// Sets the working directory for the child process
    pub fn current_dir(&mut self, dir: &str) -> &mut Self {
        self.dir = Some(dir.to_owned());
        self
    }

    /// Executes the command as a child process, and extracts its status, stdout, stderr.
    pub fn spawn(&mut self) -> io::Result<CommandRun> {
        match &self.program {
            None => Err(io::Error::new(io::ErrorKind::InvalidInput, "")),
            Some(program) => {
                let mut process = process::Command::new(program)
                    .stdout(process::Stdio::piped())
                    .stderr(process::Stdio::piped())
                    .spawn()?;
                let status = process.wait()?;
                let mut stdout = String::new();
                process.stdout.unwrap().read_to_string(&mut stdout)?;
                let mut stderr = String::new();
                process.stderr.unwrap().read_to_string(&mut stderr)?;
                Ok(CommandRun {
                    status,
                    stdout,
                    stderr
                })
            }
        }
    }
}

/// The result of a command execution if managed to run the child process
pub struct CommandRun {
    pub status: process::ExitStatus,
    pub stdout: String,
    pub stderr: String
}