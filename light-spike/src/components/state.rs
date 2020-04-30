use crate::prelude::*;

#[derive(Debug)]
pub struct State {
    pub trusted_store_reader: TSReader,
    pub trusted_store_writer: TSReadWriter,
    // valid_store_reader: TSReader,
    // valid_store_writer: TSReaderWriter,
    // fetched_store_reader: TSReader,
    // fetched_store_writer: TSReaderWriter,
}

impl State {
    pub fn trusted_store_reader(&self) -> TSReader {
        self.trusted_store_reader.clone()
    }

    pub fn add_trusted_states(&mut self, trusted_states: Vec<TrustedState>) {
        for trusted_state in trusted_states {
            self.trusted_store_writer.add(trusted_state);
        }
    }

    pub fn add_valid_light_block(&mut self, _light_block: LightBlock) {
        // self.valid_store_writer.add(light_block);
    }

    pub fn add_fetched_light_block(&mut self, _light_block: LightBlock) {
        // self.fetched_store_writer.add(light_block);
    }
}
