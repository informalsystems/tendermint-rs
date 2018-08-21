pub const AMINO_NAME: &'static str = "tendermint/kms/PoisonPillMsg";

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/kms/PoisonPillMsg"]
pub struct PoisonPillMsg {}
