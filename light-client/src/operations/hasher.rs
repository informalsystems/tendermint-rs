use crate::types::{Header, ValidatorSet};

use tendermint::amino_types::{message::AminoMessage, BlockId, ConsensusVersion, TimeMsg};
use tendermint::merkle;
use tendermint::Hash;

pub trait Hasher: Send {
    fn hash_header(&self, header: &Header) -> Hash;
    fn hash_validator_set(&self, validator_set: &ValidatorSet) -> Hash;
}

#[derive(Copy, Clone, Debug)]
pub struct ProdHasher;

impl Hasher for ProdHasher {
    fn hash_header(&self, header: &Header) -> Hash {
        amino_hash(header)
    }

    /// Compute the Merkle root of the validator set
    fn hash_validator_set(&self, validator_set: &ValidatorSet) -> Hash {
        let validator_bytes: Vec<Vec<u8>> = validator_set
            .validators()
            .iter()
            .map(|validator| validator.hash_bytes())
            .collect();

        Hash::Sha256(merkle::simple_hash_from_byte_vectors(validator_bytes))
    }
}

fn amino_hash(header: &Header) -> Hash {
    // Note that if there is an encoding problem this will
    // panic (as the golang code would):
    // https://github.com/tendermint/tendermint/blob/134fe2896275bb926b49743c1e25493f6b24cc31/types/block.go#L393
    // https://github.com/tendermint/tendermint/blob/134fe2896275bb926b49743c1e25493f6b24cc31/types/encoding_helper.go#L9:6

    let mut fields_bytes: Vec<Vec<u8>> = Vec::with_capacity(16);
    fields_bytes.push(AminoMessage::bytes_vec(&ConsensusVersion::from(
        &header.version,
    )));
    fields_bytes.push(bytes_enc(header.chain_id.as_bytes()));
    fields_bytes.push(encode_varint(header.height.into()));
    fields_bytes.push(AminoMessage::bytes_vec(&TimeMsg::from(header.time)));
    fields_bytes.push(
        header
            .last_block_id
            .as_ref()
            .map_or(vec![], |id| AminoMessage::bytes_vec(&BlockId::from(id))),
    );
    fields_bytes.push(header.last_commit_hash.as_ref().map_or(vec![], encode_hash));
    fields_bytes.push(header.data_hash.as_ref().map_or(vec![], encode_hash));
    fields_bytes.push(encode_hash(&header.validators_hash));
    fields_bytes.push(encode_hash(&header.next_validators_hash));
    fields_bytes.push(encode_hash(&header.consensus_hash));
    fields_bytes.push(bytes_enc(&header.app_hash));
    fields_bytes.push(
        header
            .last_results_hash
            .as_ref()
            .map_or(vec![], encode_hash),
    );
    fields_bytes.push(header.evidence_hash.as_ref().map_or(vec![], encode_hash));
    fields_bytes.push(bytes_enc(header.proposer_address.as_bytes()));

    Hash::Sha256(merkle::simple_hash_from_byte_vectors(fields_bytes))
}

fn bytes_enc(bytes: &[u8]) -> Vec<u8> {
    let mut chain_id_enc = vec![];
    prost_amino::encode_length_delimiter(bytes.len(), &mut chain_id_enc).unwrap();
    chain_id_enc.append(&mut bytes.to_vec());
    chain_id_enc
}

fn encode_hash(hash: &Hash) -> Vec<u8> {
    bytes_enc(hash.as_bytes())
}

fn encode_varint(val: u64) -> Vec<u8> {
    let mut val_enc = vec![];
    prost_amino::encoding::encode_varint(val, &mut val_enc);
    val_enc
}
