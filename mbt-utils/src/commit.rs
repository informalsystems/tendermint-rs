use gumdrop::Options;
use serde::Deserialize;
use signatory::ed25519;
use signatory::ed25519::SIGNATURE_SIZE;
use signatory::signature::Signature as _;
use signatory::signature::Signer;
use signatory_dalek::Ed25519Signer;
use simple_error::*;

use tendermint::signature::Signature;
use tendermint::vote::{Type, Vote};
use tendermint::{amino_types, block, lite, vote};

use crate::header::Header;
use crate::helpers::*;
use crate::generator::Generator;
use crate::validator::Validator;

#[derive(Debug, Options, Deserialize)]
pub struct Commit {
    #[options(help = "header (required)", parse(try_from_str = "parse_as::<Header>"))]
    pub header: Option<Header>,
    #[options(help = "commit round (default: 1)")]
    pub round: Option<u64>,
}

impl Commit {
    pub fn new(header: &Header) -> Self {
        Commit {
            header: Some(header.clone()),
            round: None,
        }
    }
    gen_setter!(round, u64);
}


impl std::str::FromStr for Commit {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let commit = match parse_as::<Commit>(s) {
            Ok(input) => input,
            Err(_) => Commit {
                header: Some(parse_as::<Header>(s)?),
                round: None,
            },
        };
        Ok(commit)
    }
}

impl Generator<block::Commit> for Commit {
    fn merge_with_default(&self, other: &Self) -> Self {
        Commit {
            header: choose_from(&self.header, &other.header),
            round: choose_from(&self.round, &other.round),
        }
    }

    fn generate(&self) -> Result<block::Commit, SimpleError> {
        if self.header.is_none() {
            bail!("header is missing")
        }
        let header = self.header.as_ref().unwrap();
        let block_header = header.generate()?;
        let block_id = block::Id::new(lite::Header::hash(&block_header), None);

        let val_sign = |(i, v): (usize, &Validator)| -> Result<block::CommitSig, SimpleError> {
            let validator = v.generate()?;
            let signer: Ed25519Signer = v.get_signer()?;
            let vote = Vote {
                vote_type: Type::Precommit,
                height: block_header.height,
                round: choose_or(self.round, 1),
                block_id: Some(block_id.clone()),
                timestamp: block_header.time,
                validator_address: validator.address,
                validator_index: i as u64,
                signature: Signature::Ed25519(
                    try_with!(ed25519::Signature::from_bytes(&[0_u8; SIGNATURE_SIZE]), "failed to construct empty ed25519 signature"),
                ),
            };
            let signed_vote = vote::SignedVote::new(
                amino_types::vote::Vote::from(&vote),
                block_header.chain_id.as_str(),
                validator.address,
                vote.signature
            );
            let sign_bytes = signed_vote.sign_bytes();
            Ok(block::CommitSig::BlockIDFlagCommit {
                validator_address: validator.address,
                timestamp: block_header.time,
                signature: Signature::Ed25519(try_with!(signer.try_sign(sign_bytes.as_slice()), "failed to sign using ed25519 signature")),
            })
        };
        let sigs = header.validators
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .map(val_sign)
            .collect::<Result<Vec<block::CommitSig>, SimpleError>>()?;

        let commit = block::Commit {
            height: block_header.height,
            round: choose_or(self.round, 1),
            block_id, // TODO do we need at least one part? //block::Id::new(hasher.hash_header(&block_header), None), //
            signatures: block::CommitSigs::new(sigs),
        };
        Ok(commit)
    }
}
