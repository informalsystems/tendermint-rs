
use signatory::ed25519::Signature;
use super::{ValidatorAddress,TendermintSign};
use amino::Amino;
use hex::encode;

pub struct Heartbeat{
    validator_address: ValidatorAddress,
    validator_index: i64,
    height: i64,
    round: i64,
    sequence:i64,
    Signature:Option<Signature>,
}

impl TendermintSign for Heartbeat{
    fn cannonicalize(self, chain_id: &str)->String{
        let value = json!({
            "@chain_id":chain_id,
            "@type":"heartbeat",
            "height":self.height,
            "round":self.round,
            "sequence":self.sequence,
            "validator_address": encode(self.validator_address.0),
            "validator_index": self.validator_index,
            });
        value.to_string()
    }
}

impl Amino for Heartbeat{
    fn serialize(self)->Vec<u8>{
        unimplemented!()
    }

    fn deserialize(self, data: &[u8]){
        unimplemented!()
    }
}    