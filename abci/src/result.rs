//! ABCI-related result types.

use thiserror::Error;

pub type Result<T> = eyre::Result<T>;

pub type ResultError = eyre::Report;

#[derive(Debug, Error)]
pub enum Error {
    #[error("protobuf decoding error")]
    ProtobufDecode(#[from] tendermint_proto::prost::DecodeError),

    #[error("protobuf encoding error")]
    ProtobufEncode(#[from] tendermint_proto::prost::EncodeError),
}
