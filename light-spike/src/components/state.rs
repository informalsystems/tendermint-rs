use crate::prelude::*;

#[derive(Debug)]
pub struct State {
    pub trusted_store_reader: StoreReader<Trusted>,
    pub trusted_store_writer: StoreReadWriter<Trusted>,
    pub untrusted_store_reader: StoreReader<Untrusted>,
    pub untrusted_store_writer: StoreReadWriter<Untrusted>,
}

impl State {
    pub fn add_trusted_state(&mut self, trusted_state: LightBlock) {
        self.trusted_store_writer.add(trusted_state);
    }

    pub fn add_trusted_states(&mut self, trusted_states: Vec<LightBlock>) {
        for trusted_state in trusted_states {
            self.add_trusted_state(trusted_state)
        }
    }
}
