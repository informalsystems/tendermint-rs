use abscissa_core::status_err;

use crate::config::LightStoreConfig;
use abscissa_core::component::{Component, Id};
use tendermint_light_client::store::memory::MemoryStore;
use tendermint_light_client::store::sled::SledStore;
use tendermint_light_client::store::LightStore;

pub trait LightStoreFactory: {
    fn create(&self, config: &LightStoreConfig) -> Box<dyn LightStore>;
}

#[derive(Debug)]
pub struct ProdLightStoreFactory;

impl ProdLightStoreFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl LightStoreFactory for ProdLightStoreFactory {
    fn create(&self, config: &LightStoreConfig) -> Box<dyn LightStore> {
        match config {
            LightStoreConfig::InMemory => Box::new(MemoryStore::new()),
            LightStoreConfig::OnDisk { db_path } => {
                let db = sled::open(db_path.clone()).unwrap_or_else(|e| {
                    status_err!("could not open database: {}", e);
                    std::process::exit(1);
                });
                Box::new(SledStore::new(db))
            }
        }
    }
}
