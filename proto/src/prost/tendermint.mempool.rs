#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(::serde::Deserialize, ::serde::Serialize)]
pub struct Txs {
    #[prost(bytes, repeated, tag="1")]
    pub txs: ::std::vec::Vec<std::vec::Vec<u8>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(::serde::Deserialize, ::serde::Serialize)]
pub struct Message {
    #[prost(oneof="message::Sum", tags="1")]
    pub sum: ::std::option::Option<message::Sum>,
}
pub mod message {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    pub enum Sum {
        #[prost(message, tag="1")]
        Txs(super::Txs),
    }
}
