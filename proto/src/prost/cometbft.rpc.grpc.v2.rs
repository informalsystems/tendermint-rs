#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseBroadcastTx {
    #[prost(message, optional, tag = "1")]
    pub check_tx: ::core::option::Option<super::super::super::abci::v2::ResponseCheckTx>,
    #[prost(message, optional, tag = "2")]
    pub deliver_tx: ::core::option::Option<
        super::super::super::abci::v2::ResponseDeliverTx,
    >,
}
