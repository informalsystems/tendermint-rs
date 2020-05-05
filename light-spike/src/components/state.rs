use crate::prelude::*;

#[derive(Debug)]
pub struct Peers {
    pub primary: Peer,
    pub witnesses: Vec<Peer>,
}

#[derive(Debug)]
pub struct State {
    pub peers: Peers,
    pub trusted_store_reader: StoreReader<Trusted>,
    pub trusted_store_writer: StoreReadWriter<Trusted>,
    pub untrusted_store_reader: StoreReader<Untrusted>,
    pub untrusted_store_writer: StoreReadWriter<Untrusted>,
}
