use crate::prelude::*;

#[derive(Debug)]
pub struct State {
    pub trusted_store_reader: StoreReader<Trusted>,
    pub trusted_store_writer: StoreReadWriter<Trusted>,
    pub untrusted_store_reader: StoreReader<Untrusted>,
    pub untrusted_store_writer: StoreReadWriter<Untrusted>,
    pub errors: Vec<Error>,
}
