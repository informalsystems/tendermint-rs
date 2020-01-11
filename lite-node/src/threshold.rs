use tendermint::lite::TrustThreshold;

pub struct TrustThresholdOneThird {}
impl TrustThreshold for TrustThresholdOneThird {}

pub struct TrustThresholdTwoThirds {}
impl TrustThreshold for TrustThresholdTwoThirds {
    fn is_enough_power(&self, signed_voting_power: u64, total_voting_power: u64) -> bool {
        signed_voting_power * 3 > total_voting_power * 2
    }
}
