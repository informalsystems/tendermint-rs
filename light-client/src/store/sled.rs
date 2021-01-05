//! Persistent store backed by an on-disk `sled` database.

pub mod utils;

use ::sled::Db as SledDb;

use crate::{
    store::sled::utils::HeightIndexedDb,
    types::{Height, LightBlock},
};

use super::{LightStore, Status};

const UNVERIFIED: &str = "unverified";
const VERIFIED: &str = "verified";
const TRUSTED: &str = "trusted";
const FAILED: &str = "failed";

/// Persistent store backed by an on-disk `sled` database.
#[derive(Debug, Clone)]
pub struct SledStore {
    unverified_db: HeightIndexedDb<LightBlock>,
    verified_db: HeightIndexedDb<LightBlock>,
    trusted_db: HeightIndexedDb<LightBlock>,
    failed_db: HeightIndexedDb<LightBlock>,
    db: SledDb,
}

impl SledStore {
    /// Create a new persistent store from a sled database
    pub fn new(db: SledDb) -> Self {
        Self {
            unverified_db: HeightIndexedDb::new(db.open_tree(UNVERIFIED).unwrap()),
            verified_db: HeightIndexedDb::new(db.open_tree(VERIFIED).unwrap()),
            trusted_db: HeightIndexedDb::new(db.open_tree(TRUSTED).unwrap()),
            failed_db: HeightIndexedDb::new(db.open_tree(FAILED).unwrap()),
            db,
        }
    }

    fn db(&self, status: Status) -> &HeightIndexedDb<LightBlock> {
        match status {
            Status::Unverified => &self.unverified_db,
            Status::Verified => &self.verified_db,
            Status::Trusted => &self.trusted_db,
            Status::Failed => &self.failed_db,
        }
    }
}

impl LightStore for SledStore {
    fn get(&self, height: Height, status: Status) -> Option<LightBlock> {
        self.db(status).get(height).ok().flatten()
    }

    fn update(&mut self, light_block: &LightBlock, status: Status) {
        let height = light_block.height();

        for other in Status::iter() {
            if status != *other {
                self.db(*other).remove(height).ok();
            }
        }

        self.db(status).insert(height, light_block).ok();
    }

    fn insert(&mut self, light_block: LightBlock, status: Status) {
        self.db(status)
            .insert(light_block.height(), &light_block)
            .ok();
    }

    fn remove(&mut self, height: Height, status: Status) {
        self.db(status).remove(height).ok();
    }

    fn latest(&self, status: Status) -> Option<LightBlock> {
        self.db(status).iter().next_back()
    }

    fn all(&self, status: Status) -> Box<dyn Iterator<Item = LightBlock>> {
        Box::new(self.db(status).iter())
    }
}
