use signatory::ed25519::Signature;

mod heartbeat;
mod proposal;
mod vote;

pub struct ValidatorAddress([u8;20]);

pub trait TendermintSign{
    fn cannonicalize(self)->String;
}

pub trait Amino{
    fn serialize(self)->Vec<u8>;
    fn deserialize(self, Vec<u8>);
}

pub use self::heartbeat::Heartbeat;