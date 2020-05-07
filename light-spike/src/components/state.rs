use crate::prelude::*;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Peers {
    pub primary: Peer,
    pub witnesses: Vec<Peer>,
}

#[derive(Debug)]
pub struct State {
    pub peers: Peers,
    pub trusted_store_reader: StoreReader<Trusted>,
    pub trusted_store_writer: StoreWriter<Trusted>,
    pub untrusted_store_reader: StoreReader<Untrusted>,
    pub untrusted_store_writer: StoreWriter<Untrusted>,
    pub verification_trace: HashMap<Height, Vec<Height>>,
}

impl State {
    pub fn trace_block(&mut self, target_height: Height, height: Height) {
        precondition!(height <= target_height);

        if height < target_height {
            self.verification_trace
                .entry(target_height)
                .or_insert_with(|| Vec::new())
                .push(height);
        }

        postcondition!(self
            .verification_trace
            .get(&target_height)
            .map_or(true, |trace| { trace.iter().all(|h| h < &target_height) }))
    }
}
