#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(::serde::Deserialize, ::serde::Serialize)]
pub struct List {
    #[prost(message, repeated, tag="1")]
    pub evidence: ::std::vec::Vec<super::types::Evidence>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(::serde::Deserialize, ::serde::Serialize)]
pub struct Info {
    #[prost(message, optional, tag="1")]
    pub evidence: ::std::option::Option<super::types::Evidence>,
    #[prost(message, optional, tag="2")]
    pub time: ::std::option::Option<super::super::google::protobuf::Timestamp>,
    #[prost(message, repeated, tag="3")]
    pub validators: ::std::vec::Vec<super::types::Validator>,
    #[prost(int64, tag="4")]
    pub total_voting_power: i64,
}
