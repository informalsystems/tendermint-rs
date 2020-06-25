pub mod utils;

use crate::{
    store::sled::utils::*,
    types::{Height, LightBlock},
};

use super::{LightStore, Status};
use ::sled::Db as SledDb;

const UNVERIFIED_PREFIX: &str = "light_store/unverified";
const VERIFIED_PREFIX: &str = "light_store/verified";
const TRUSTED_PREFIX: &str = "light_store/trusted";
const FAILED_PREFIX: &str = "light_store/failed";

/// Persistent store backed by an on-disk `sled` database.
#[derive(Debug, Clone)]
pub struct SledStore {
    db: SledDb,
    unverified_db: KeyValueDb<Height, LightBlock>,
    verified_db: KeyValueDb<Height, LightBlock>,
    trusted_db: KeyValueDb<Height, LightBlock>,
    failed_db: KeyValueDb<Height, LightBlock>,
}

impl SledStore {
    pub fn new(db: SledDb) -> Self {
        Self {
            db,
            unverified_db: KeyValueDb::new(UNVERIFIED_PREFIX),
            verified_db: KeyValueDb::new(VERIFIED_PREFIX),
            trusted_db: KeyValueDb::new(TRUSTED_PREFIX),
            failed_db: KeyValueDb::new(FAILED_PREFIX),
        }
    }

    fn db(&self, status: Status) -> &KeyValueDb<Height, LightBlock> {
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
        self.db(status).get(&self.db, &height).ok().flatten()
    }

    fn update(&mut self, light_block: &LightBlock, status: Status) {
        let height = light_block.height();

        for other in Status::iter() {
            if status != *other {
                self.db(*other).remove(&self.db, &height).ok();
            }
        }

        self.db(status).insert(&self.db, &height, light_block).ok();
    }

    fn insert(&mut self, light_block: LightBlock, status: Status) {
        self.db(status)
            .insert(&self.db, &light_block.height(), &light_block)
            .ok();
    }

    fn remove(&mut self, height: Height, status: Status) {
        self.db(status).remove(&self.db, &height).ok();
    }

    fn highest(&self, status: Status) -> Option<LightBlock> {
        self.db(status).iter(&self.db).next_back()
    }

    fn all(&self, status: Status) -> Box<dyn Iterator<Item = LightBlock>> {
        Box::new(self.db(status).iter(&self.db))
    }
}
