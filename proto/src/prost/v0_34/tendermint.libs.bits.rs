#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BitArray {
    #[prost(int64, tag="1")]
    pub bits: i64,
    #[prost(uint64, repeated, tag="2")]
    pub elems: ::prost::alloc::vec::Vec<u64>,
}
