use simple_error::*;

use crate::validator::generate_validator_set;
use crate::{Commit, Generator, Header};
use tendermint::block::signed_header::SignedHeader;
use tendermint::node::Id as PeerId;
use tendermint::validator::Info;
use tendermint::validator::Set as ValidatorSet;

/// A light block is the core data structure used by the light client.
/// It records everything the light client needs to know about a block.
#[derive(Clone, Debug, PartialEq)]
pub struct LightBlock {
    /// Header and commit of this block
    pub signed_header: SignedHeader,
    /// Validator set at the block height
    pub validators: ValidatorSet,
    /// Validator set at the next block height
    pub next_validators: ValidatorSet,
    /// The peer ID of the node that provided this block
    pub provider: PeerId,
}

impl LightBlock {
    /// Constructs a new light block
    pub fn new(
        signed_header: SignedHeader,
        validators: ValidatorSet,
        next_validators: ValidatorSet,
        provider: PeerId,
    ) -> LightBlock {
        Self {
            signed_header,
            validators,
            next_validators,
            provider,
        }
    }

    pub fn generate_default(
        val_ids: Vec<&str>,
        peer_id: PeerId,
    ) -> Result<LightBlock, SimpleError> {
        let (validator_set, raw_vals) = match generate_validator_set(val_ids) {
            Err(e) => bail!("Failed to generate validator set with error: {}", e),
            Ok(v) => v,
        };

        let raw_header = Header::new(&raw_vals);
        let raw_commit = Commit::new(raw_header.clone(), 1);
        let signed_header = match generate_signed_header(raw_header, raw_commit) {
            Err(e) => bail!("Failed to generate signed header with error: {}", e),
            Ok(sh) => sh,
        };

        let light_block = LightBlock::new(signed_header, validator_set.clone(), validator_set, peer_id);
        Ok(light_block)
    }

    pub fn generate_with(
        raw_header: Header,
        raw_commit: Commit,
        raw_vals: Vec<Info>,
        peer_id: PeerId,
    ) -> Result<LightBlock, SimpleError> {
        let signed_header = match generate_signed_header(raw_header, raw_commit) {
            Err(e) => bail!("Failed to generate signed header with error: {}", e),
            Ok(sh) => sh,
        };

        let validator_set = ValidatorSet::new(raw_vals);

        let light_block = LightBlock::new(signed_header, validator_set.clone(), validator_set, peer_id);
        Ok(light_block)
    }
}

pub fn generate_signed_header(
    raw_header: Header,
    raw_commit: Commit,
) -> Result<SignedHeader, SimpleError> {
    let header = match raw_header.generate() {
        Err(e) => bail!("Failed to generate header with error: {}", e),
        Ok(h) => h,
    };

    let commit = match raw_commit.generate() {
        Err(e) => bail!("Failed to generate commit with error: {}", e),
        Ok(c) => c,
    };

    Ok(SignedHeader { header, commit })
}
