#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Txs {
    #[prost(bytes, repeated, tag="1")]
    pub txs: ::std::vec::Vec<std::vec::Vec<u8>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(oneof="message::Sum", tags="1")]
    pub sum: ::std::option::Option<message::Sum>,
}
pub mod message {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag="1")]
        Txs(super::Txs),
    }
}
