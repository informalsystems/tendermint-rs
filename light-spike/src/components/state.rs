use crate::prelude::*;

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ErrorKind {
    #[error("I/O error")]
    Io(#[from] IoError),
    #[error("no initial trusted state")]
    NoInitialTrustedState,
}

pub type Error = anomaly::Error<ErrorKind>;

#[derive(Debug)]
pub struct State {
    pub trusted_store_reader: StoreReader<Trusted>,
    pub trusted_store_writer: StoreReadWriter<Trusted>,
    pub untrusted_store_reader: StoreReader<Untrusted>,
    pub untrusted_store_writer: StoreReadWriter<Untrusted>,
    pub errors: Vec<Error>,
}
