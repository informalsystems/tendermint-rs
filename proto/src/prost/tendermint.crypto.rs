#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Proof {
    #[prost(int64, tag="1")]
    #[serde(with = "crate::serializers::from_str")]
    pub total: i64,
    #[prost(int64, tag="2")]
    #[serde(with = "crate::serializers::from_str")]
    pub index: i64,
    #[prost(bytes="vec", tag="3")]
    #[serde(with = "crate::serializers::bytes::base64string")]
    pub leaf_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", repeated, tag="4")]
    #[serde(with = "crate::serializers::bytes::vec_base64string")]
    pub aunts: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValueOp {
    /// Encoded in ProofOp.Key.
    #[prost(bytes="vec", tag="1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    /// To encode in ProofOp.Data
    #[prost(message, optional, tag="2")]
    pub proof: ::core::option::Option<Proof>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DominoOp {
    #[prost(string, tag="1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub input: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub output: ::prost::alloc::string::String,
}
/// ProofOp defines an operation used for calculating Merkle root
/// The data could be arbitrary format, providing nessecary data
/// for example neighbouring node hash
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProofOp {
    #[prost(string, tag="1")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="2")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="3")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
/// ProofOps is Merkle proof defined by the list of ProofOps
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProofOps {
    #[prost(message, repeated, tag="1")]
    pub ops: ::prost::alloc::vec::Vec<ProofOp>,
}
/// PublicKey defines the keys available for use with Tendermint Validators
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublicKey {
    #[prost(oneof="public_key::Sum", tags="1, 2, 3")]
    pub sum: ::core::option::Option<public_key::Sum>,
}
/// Nested message and enum types in `PublicKey`.
pub mod public_key {
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[serde(tag = "type", content = "value")]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(bytes, tag="1")]
        #[serde(rename = "tendermint/PubKeyEd25519", with = "crate::serializers::bytes::base64string")]
        Ed25519(::prost::alloc::vec::Vec<u8>),
        #[prost(bytes, tag="2")]
        #[serde(rename = "tendermint/PubKeySecp256k1", with = "crate::serializers::bytes::base64string")]
        Secp256k1(::prost::alloc::vec::Vec<u8>),
        #[prost(bytes, tag="3")]
        Sr25519(::prost::alloc::vec::Vec<u8>),
    }
}
