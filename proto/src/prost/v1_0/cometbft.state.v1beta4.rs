#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbciResponsesInfo {
    /// Retains the responses of the legacy ABCI calls during block processing.
    #[prost(message, optional, tag = "1")]
    pub legacy_abci_responses: ::core::option::Option<
        super::v1beta3::LegacyAbciResponses,
    >,
    #[prost(int64, tag = "2")]
    pub height: i64,
    #[prost(message, optional, tag = "3")]
    pub finalize_block: ::core::option::Option<
        super::super::abci::v1beta4::FinalizeBlockResponse,
    >,
}
