/// Consensus captures the consensus rules for processing a block in the blockchain,
/// including all blockchain data structures and the rules of the application's
/// state transition machine.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Consensus {
    #[prost(uint64, tag="1")]
    #[serde(with = "crate::serializers::from_str")]
    pub block: u64,
    #[prost(uint64, tag="2")]
    #[serde(with = "crate::serializers::from_str", default)]
    pub app: u64,
}
