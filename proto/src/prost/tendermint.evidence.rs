#[derive(Clone, PartialEq, ::prost::Message)]
pub struct List {
    #[prost(message, repeated, tag="1")]
    pub evidence: ::std::vec::Vec<super::types::Evidence>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Info {
    #[prost(bool, tag="1")]
    pub committed: bool,
    #[prost(int64, tag="2")]
    pub priority: i64,
    #[prost(message, optional, tag="3")]
    pub evidence: ::std::option::Option<super::types::Evidence>,
}
