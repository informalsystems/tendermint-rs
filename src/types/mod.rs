use signatory::ed25519::Signature;

mod heartbeat;
mod proposal;
mod vote;

pub struct ValidatorAddress([u8;20]);

pub trait TendermintSign{
    fn cannonicalize(self, chain_id:&str)->String;
}


pub use self::heartbeat::Heartbeat;