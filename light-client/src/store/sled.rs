//! Persistent store backed by an on-disk `sled` database.

pub mod utils;

use std::path::Path;

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
}

impl SledStore {
    /// Open a sled database and create a new persistent store from it.
    pub fn open(db: impl AsRef<Path>) -> Result<Self, sled::Error> {
        Ok(Self::new(sled::open(db)?))
    }

    /// Create a new persistent store from a sled database that is already open.
    pub fn new(db: sled::Db) -> Self {
        Self {
            unverified_db: HeightIndexedDb::new(db.open_tree(UNVERIFIED).unwrap()),
            verified_db: HeightIndexedDb::new(db.open_tree(VERIFIED).unwrap()),
            trusted_db: HeightIndexedDb::new(db.open_tree(TRUSTED).unwrap()),
            failed_db: HeightIndexedDb::new(db.open_tree(FAILED).unwrap()),
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

    fn highest(&self, status: Status) -> Option<LightBlock> {
        self.db(status).iter().next_back()
    }

    fn lowest(&self, status: Status) -> Option<LightBlock> {
        self.db(status).iter().next()
    }

    fn all(&self, status: Status) -> Box<dyn Iterator<Item = LightBlock>> {
        Box::new(self.db(status).iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tendermint_testgen::{light_block::TmLightBlock as TGLightBlock, Generator, LightChain};

    #[test]
    fn highest_returns_latest_block() {
        with_blocks(10, |mut db, blocks| {
            let initial_block = blocks[0].clone();
            db.insert(initial_block.clone(), Status::Verified);
            assert_eq!(db.lowest(Status::Verified).as_ref(), Some(&initial_block));

            for block in blocks.into_iter().skip(1) {
                db.insert(block, Status::Verified);
                assert_eq!(db.lowest(Status::Verified).as_ref(), Some(&initial_block));
            }
        })
    }

    #[test]
    fn lowest_returns_earliest_block() {
        with_blocks(10, |mut db, blocks| {
            for block in blocks {
                db.insert(block.clone(), Status::Verified);
                assert_eq!(db.highest(Status::Verified), Some(block));
            }
        })
    }

    fn with_blocks(height: u64, f: impl FnOnce(SledStore, Vec<LightBlock>)) {
        let tmp_dir = tempdir().unwrap();
        let db = SledStore::open(tmp_dir).unwrap();

        let chain = LightChain::default_with_length(height);
        let blocks = chain
            .light_blocks
            .into_iter()
            .map(|lb| lb.generate().unwrap())
            .map(testgen_to_lb)
            .collect::<Vec<_>>();

        f(db, blocks)
    }

    fn testgen_to_lb(tm_lb: TGLightBlock) -> LightBlock {
        LightBlock {
            signed_header: tm_lb.signed_header,
            validators: tm_lb.validators,
            next_validators: tm_lb.next_validators,
            provider: tm_lb.provider,
        }
    }
}
