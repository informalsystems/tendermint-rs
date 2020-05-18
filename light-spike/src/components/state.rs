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
    pub light_store: Box<dyn LightStore>,
    pub verification_trace: HashMap<Height, HashSet<Height>>,
}

impl State {
    #[pre(height <= target_height)]
    pub fn trace_block(&mut self, target_height: Height, height: Height) {
        self.verification_trace
            .entry(target_height)
            .or_insert_with(|| HashSet::new())
            .insert(height);
    }

    #[pre(self.verification_trace.contains_key(&target_height))]
    pub fn get_trace(&self, target_height: Height) -> Vec<LightBlock> {
        let mut trace = self
            .verification_trace
            .get(&target_height)
            .unwrap_or(&HashSet::new())
            .iter()
            .flat_map(|h| self.light_store.get(*h, VerifiedStatus::Verified))
            .collect::<Vec<_>>();

        trace.sort_by_key(|lb| lb.height());
        trace.reverse();
        trace
    }
}
