use crate::prelude::*;

use std::collections::{HashMap, HashSet};

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
    pub verification_trace: HashMap<Height, HashSet<Height>>,
}

impl State {
    pub fn trace_block(&mut self, target_height: Height, height: Height) {
        precondition!(height <= target_height);

        self.verification_trace
            .entry(target_height)
            .or_insert_with(|| HashSet::new())
            .insert(height);

        postcondition!(self
            .verification_trace
            .get(&target_height)
            .map_or(true, |trace| { trace.iter().all(|h| h <= &target_height) }))
    }

    pub fn get_trace(&self, target_height: Height) -> Vec<LightBlock> {
        precondition!(self.verification_trace.contains_key(&target_height));

        let mut trace = self
            .verification_trace
            .get(&target_height)
            .unwrap_or(&HashSet::new())
            .iter()
            .flat_map(|h| self.trusted_store_reader.get(*h))
            .collect::<Vec<_>>();

        trace.sort_by_key(|lb| lb.height());
        trace.reverse();
        trace
    }
}
