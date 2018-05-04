use chrono::{DateTime,Utc};
use signatory::ed25519::{Signature, SIGNATURE_SIZE};
use super::{ValidatorAddress,TendermintSign, BlockID};
use hex::encode_upper;

pub struct Vote{
    validator_address:ValidatorAddress,
    validator_index: i64,
    height: i64,
    round: i64,
    timestamp: DateTime<Utc>,
    block_id: BlockID,
    signature: Signature
}

impl TendermintSign for Vote{
    fn cannonicalize(self, chain_id: &str)->String{
        unimplemented!()
    }
}