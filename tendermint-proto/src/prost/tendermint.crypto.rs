#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Proof {
    #[prost(int64, tag="1")]
    pub total: i64,
    #[prost(int64, tag="2")]
    pub index: i64,
    #[prost(bytes, tag="3")]
    pub leaf_hash: std::vec::Vec<u8>,
    #[prost(bytes, repeated, tag="4")]
    pub aunts: ::std::vec::Vec<std::vec::Vec<u8>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValueOp {
    /// Encoded in ProofOp.Key.
    #[prost(bytes, tag="1")]
    pub key: std::vec::Vec<u8>,
    /// To encode in ProofOp.Data
    #[prost(message, optional, tag="2")]
    pub proof: ::std::option::Option<Proof>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DominoOp {
    #[prost(string, tag="1")]
    pub key: std::string::String,
    #[prost(string, tag="2")]
    pub input: std::string::String,
    #[prost(string, tag="3")]
    pub output: std::string::String,
}
/// ProofOp defines an operation used for calculating Merkle root
/// The data could be arbitrary format, providing nessecary data
/// for example neighbouring node hash
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProofOp {
    #[prost(string, tag="1")]
    pub r#type: std::string::String,
    #[prost(bytes, tag="2")]
    pub key: std::vec::Vec<u8>,
    #[prost(bytes, tag="3")]
    pub data: std::vec::Vec<u8>,
}
/// ProofOps is Merkle proof defined by the list of ProofOps
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProofOps {
    #[prost(message, repeated, tag="1")]
    pub ops: ::std::vec::Vec<ProofOp>,
}
/// PublicKey defines the keys available for use with Tendermint Validators
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublicKey {
    #[prost(oneof="public_key::Sum", tags="1")]
    pub sum: ::std::option::Option<public_key::Sum>,
}
pub mod public_key {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(bytes, tag="1")]
        Ed25519(std::vec::Vec<u8>),
    }
}
/// PrivateKey defines the keys available for use with Tendermint Validators
/// WARNING PrivateKey is used for internal purposes only
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrivateKey {
    #[prost(oneof="private_key::Sum", tags="1")]
    pub sum: ::std::option::Option<private_key::Sum>,
}
pub mod private_key {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(bytes, tag="1")]
        Ed25519(std::vec::Vec<u8>),
    }
}
