//! [`lite::Header`] implementation for [`block::Header`].

use crate::amino_types::{message::AminoMessage, BlockId, ConsensusVersion, TimeMsg};
use crate::merkle::simple_hash_from_byte_vectors;
use crate::Hash;
use crate::{block, lite, Time};

impl lite::Header for block::Header {
    type Time = Time;

    fn height(&self) -> block::Height {
        self.height
    }

    fn bft_time(&self) -> Time {
        self.time
    }

    fn validators_hash(&self) -> Hash {
        self.validators_hash
    }

    fn next_validators_hash(&self) -> Hash {
        self.next_validators_hash
    }

    fn hash(&self) -> Hash {
        // Note that if there is an encoding problem this will
        // panic (as the golang code would):
        // https://github.com/tendermint/tendermint/blob/134fe2896275bb926b49743c1e25493f6b24cc31/types/block.go#L393
        // https://github.com/tendermint/tendermint/blob/134fe2896275bb926b49743c1e25493f6b24cc31/types/encoding_helper.go#L9:6

        let mut fields_bytes: Vec<Vec<u8>> = Vec::with_capacity(16);
        fields_bytes.push(AminoMessage::bytes_vec(&ConsensusVersion::from(
            &self.version,
        )));
        fields_bytes.push(bytes_enc(self.chain_id.as_bytes()));
        fields_bytes.push(encode_varint(self.height.value()));
        fields_bytes.push(AminoMessage::bytes_vec(&TimeMsg::from(self.time)));
        fields_bytes.push(encode_varint(self.num_txs));
        fields_bytes.push(encode_varint(self.total_txs));
        fields_bytes.push(
            self.last_block_id
                .as_ref()
                .map_or(vec![], |id| AminoMessage::bytes_vec(&BlockId::from(id))),
        );
        fields_bytes.push(self.last_commit_hash.as_ref().map_or(vec![], encode_hash));
        fields_bytes.push(self.data_hash.as_ref().map_or(vec![], encode_hash));
        fields_bytes.push(encode_hash(&self.validators_hash));
        fields_bytes.push(encode_hash(&self.next_validators_hash));
        fields_bytes.push(encode_hash(&self.consensus_hash));
        fields_bytes.push(bytes_enc(&self.app_hash));
        fields_bytes.push(self.last_results_hash.as_ref().map_or(vec![], encode_hash));
        fields_bytes.push(self.evidence_hash.as_ref().map_or(vec![], encode_hash));
        fields_bytes.push(bytes_enc(self.proposer_address.as_bytes()));

        Hash::Sha256(simple_hash_from_byte_vectors(fields_bytes))
    }
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
