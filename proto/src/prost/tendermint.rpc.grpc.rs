//----------------------------------------
// Request types

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestPing {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestBroadcastTx {
    #[prost(bytes, tag="1")]
    pub tx: std::vec::Vec<u8>,
}
//----------------------------------------
// Response types

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponsePing {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseBroadcastTx {
    #[prost(message, optional, tag="1")]
    pub check_tx: ::std::option::Option<super::super::abci::ResponseCheckTx>,
    #[prost(message, optional, tag="2")]
    pub deliver_tx: ::std::option::Option<super::super::abci::ResponseDeliverTx>,
}
