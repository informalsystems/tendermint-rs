use simple_error::*;
use tendermint::SignedHeader;
use tendermint_light_client::types::{LightBlock, ValidatorSet, SignedHeader, PeerId};

use crate::{Commit, Header, Validator, Generator};
use crate::validator::{generate_validators, generate_validator_set};
use tendermint::validator::Info;
use tendermint_light_client::predicates::verify;

pub fn generate_default_light_block(
    val_ids: Vec<&str>,
    peer_id: PeerId
) -> Result<LightBlock, SimpleError> {
    let (validator_set, raw_vals) =  match generate_validator_set(val_ids) {
        Err(e) => bail!("Failed to generate validator set with error: {}", e),
        Ok(v) => v,
    };

    let raw_header = Header::new(raw_vals);
    let raw_commit = Commit::new(raw_header.clone(), 1);
    let signed_header = match generate_signed_header(raw_header, raw_commit) {
        Err(e) => bail!("Failed to generate signed header with error: {}", e),
        Ok(sh) => sh,
    };

    let light_block = LightBlock::new(
        signed_header,
        validator_set.clone(),
        validator_set,
        peer_id
    );
    Ok(light_block)
}

pub fn generate_light_block_with(
    raw_header: Header,
    raw_commit: Commit,
    raw_vals: Vec<Info>
) -> Result<LightBlock, SimpleError> {
    let signed_header = match generate_signed_header(raw_header, raw_commit) {
        Err(e) => bail!("Failed to generate signed header with error: {}", e),
        Ok(sh) => sh,
    };

    let validator_set = ValidatorSet::new(raw_vals);

    let light_block = LightBlock::new(
        signed_header,
        validator_set.clone(),
        validator_set,
        peer_id
    );
    Ok(light_block)
}

pub fn generate_signed_header(raw_header: Header, raw_commit: Commit) -> Result<SignedHeader, SimpleError> {
    let header = match raw_header.generate() {
        Err(e) => bail!("Failed to generate header with error: {}", e),
        Ok(h) => h,
    };

    let commit = match raw_commit.generate() {
        Err(e) => bail!("Failed to generate commit with error: {}", e),
        Ok(c) => c,
    };

    Ok(SignedHeader{
        header,
        commit
    })
}