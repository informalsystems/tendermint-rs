use derive_more::Display;
use serde::{Deserialize, Serialize};

use tendermint::{
    account::Id as AccountId, block::header::Version as HeaderVersion,
    block::signed_header::SignedHeader as TMSignedHeader, block::Id as BlockId,
    chain::Id as ChainId, lite::Header as _, Time,
};

use crate::prelude::*;

pub use tendermint::{hash::Hash, lite::Height};

#[derive(Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct VerificationOptions {
    pub trust_threshold: TrustThreshold,
    pub trusting_period: Duration,
    pub now: Time,
}

impl VerificationOptions {
    pub fn with_now(&self, now: Time) -> Self {
        Self {
            now,
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct Header {
    pub version: HeaderVersion,
    pub chain_id: ChainId,
    pub height: Height,
    pub bft_time: Time,
    pub validators_hash: Hash,
    pub next_validators_hash: Hash,
    pub proposer_address: AccountId,
    pub evidence_hash: Option<Hash>,
    pub last_results_hash: Option<Hash>,
    pub last_block_id: Option<BlockId>,
    pub last_commit_hash: Option<Hash>,
    pub data_hash: Option<Hash>,
    pub consensus_hash: Hash,
    pub app_hash: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct ValidatorSet {
    pub hash: Hash,
}

impl From<std::vec::Vec<tendermint::validator::Info>> for ValidatorSet {
    fn from(_vis: std::vec::Vec<tendermint::validator::Info>) -> Self {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct Commit {
    pub header_hash: Hash,
    pub commit: tendermint::block::Commit,
}

#[derive(Copy, Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct TrustThreshold {
    pub numerator: u64,
    pub denominator: u64,
}

#[derive(Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct SignedHeader {
    pub header: Header,
    pub commit: Commit,
    pub validators: ValidatorSet,
    pub validators_hash: Hash,
}

impl From<TMSignedHeader> for SignedHeader {
    fn from(sh: TMSignedHeader) -> Self {
        let validators = ValidatorSet {
            hash: sh.header.validators_hash(),
        };

        Self {
            header: Header {
                height: sh.header.height().into(),
                bft_time: sh.header.bft_time(),
                validators_hash: sh.header.validators_hash(),
                next_validators_hash: sh.header.next_validators_hash(),
                version: sh.header.version,
                chain_id: sh.header.chain_id,
                proposer_address: sh.header.proposer_address,
                evidence_hash: sh.header.evidence_hash,
                last_results_hash: sh.header.last_results_hash,
                last_block_id: sh.header.last_block_id,
                last_commit_hash: sh.header.last_commit_hash,
                data_hash: sh.header.data_hash,
                consensus_hash: sh.header.consensus_hash,
                app_hash: sh.header.app_hash,
            },
            commit: Commit {
                header_hash: sh.commit.block_id.hash,
                commit: sh.commit,
            },
            validators: validators.clone(),
            validators_hash: validators.hash,
        }
    }
}

pub type TrustedState = LightBlock;

#[derive(Clone, Debug, Display, PartialEq, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct LightBlock {
    pub height: Height,
    pub signed_header: SignedHeader,
    pub validators: ValidatorSet,
    pub next_validators: ValidatorSet,
    pub provider: Peer,
}

impl LightBlock {
    pub fn header(&self) -> &Header {
        &self.signed_header.header
    }

    pub fn from_signed_header(sh: TMSignedHeader, provider: Peer) -> LightBlock {
        let height = sh.header.height.into();

        let validators = ValidatorSet {
            hash: sh.header.validators_hash(),
        };

        let next_validators = ValidatorSet {
            hash: sh.header.next_validators_hash(),
        };

        let signed_header = sh.into();

        Self {
            height,
            signed_header,
            validators,
            next_validators,
            provider,
        }
    }
}
