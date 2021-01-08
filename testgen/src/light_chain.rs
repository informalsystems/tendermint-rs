use crate::{light_block::LightBlock, Generator};
use tendermint::block::{self, Height};
use tendermint::chain::Info;

use std::convert::{TryFrom, TryInto};

#[derive(Clone, Debug)]
pub struct LightChain {
    pub info: Info,
    pub light_blocks: Vec<LightBlock>,
}

impl LightChain {
    pub fn new(info: Info, light_blocks: Vec<LightBlock>) -> Self {
        LightChain { info, light_blocks }
    }

    // TODO: make this fn more usable
    // TODO: like how does someone generate a chain with different validators at each height
    pub fn default_with_length(num: u64) -> Self {
        let mut last_block = LightBlock::new_default(1);
        let mut light_blocks: Vec<LightBlock> = vec![last_block.clone()];

        for _i in 2..=num {
            // add "next" light block to the vector
            last_block = last_block.next();
            light_blocks.push(last_block.clone());
        }

        let id = last_block.chain_id().parse().unwrap();
        let height = last_block.height().try_into().unwrap();
        let last_block_hash = last_block.header.map(|h| h.generate().unwrap().hash());
        let last_block_id = last_block_hash.map(|hash| block::Id {
            hash,
            part_set_header: Default::default(),
        });

        let info = Info {
            id,
            height,
            last_block_id,
            // TODO: Not sure yet what this time means
            time: None,
        };

        Self::new(info, light_blocks)
    }

    /// expects at least one LightBlock in the Chain
    pub fn advance_chain(&mut self) -> &LightBlock {
        let last_light_block = self
            .light_blocks
            .last()
            .expect("Cannot find testgen light block");

        let new_light_block = last_light_block.next();

        self.info.height = Height::try_from(new_light_block.height())
            .expect("failed to convert from u64 to Height");

        let last_block_id_hash = new_light_block
            .header
            .as_ref()
            .expect("missing header in new light block")
            .generate()
            .expect("failed to generate header")
            .hash();

        self.info.last_block_id = Some(block::Id {
            hash: last_block_id_hash,
            part_set_header: Default::default(),
        });

        self.light_blocks.push(new_light_block);
        self.light_blocks.last().unwrap() // safe because of push above
    }

    /// fetches a block from LightChain at a certain height
    /// it returns None if a block does not exist for the target_height
    pub fn block(&self, target_height: u64) -> Option<&LightBlock> {
        self.light_blocks
            .iter()
            .find(|lb| lb.height() == target_height)
    }

    /// fetches a mutable block from LightChain at a certain height
    /// it returns None if a block does not exist for the target_height
    pub fn block_mut(&mut self, target_height: u64) -> Option<&mut LightBlock> {
        self.light_blocks
            .iter_mut()
            .find(|lb| lb.height() == target_height)
    }

    /// fetches the latest block from LightChain
    pub fn latest_block(&self) -> &LightBlock {
        self.light_blocks
            .last()
            .expect("cannot find last light block")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advance_chain() {
        let mut light_chain = LightChain::default_with_length(1);
        let advance_1 = light_chain.advance_chain();

        assert_eq!(2, advance_1.height());
        assert_eq!(2, light_chain.info.height.value());

        let advance_2 = light_chain.advance_chain();

        assert_eq!(3, advance_2.height());
        assert_eq!(3, light_chain.info.height.value());
    }

    #[test]
    fn test_block() {
        let mut light_chain = LightChain::default_with_length(1);
        let first_block = light_chain.block(1);
        assert_eq!(1, first_block.unwrap().height());

        light_chain.advance_chain();
        let second_block = light_chain.block(2);
        assert_eq!(2, second_block.unwrap().height());
    }

    #[test]
    fn test_latest_block() {
        let mut light_chain = LightChain::default_with_length(1);
        let first_block = light_chain.latest_block();
        assert_eq!(1, first_block.height());

        light_chain.advance_chain();
        let second_block = light_chain.latest_block();
        assert_eq!(2, second_block.height());
    }

    #[test]
    fn test_light_chain_with_length() {
        const CHAIN_HEIGHT: u64 = 10;

        let chain = LightChain::default_with_length(CHAIN_HEIGHT);

        let blocks = chain
            .light_blocks
            .into_iter()
            .flat_map(|lb| lb.generate())
            .collect::<Vec<_>>();

        // we have as many blocks as the height of the chain
        assert_eq!(blocks.len(), chain.info.height.value() as usize);
        assert_eq!(blocks.len(), CHAIN_HEIGHT as usize);

        let first_block = blocks.first().unwrap();
        let last_block = blocks.last().unwrap();

        // the first block is at height 1
        assert_eq!(first_block.signed_header.header.height.value(), 1);

        // the first block does not have a last_block_id
        assert!(first_block.signed_header.header.last_block_id.is_none());

        // the last block is at the chain height
        assert_eq!(last_block.signed_header.header.height, chain.info.height);

        for i in 1..blocks.len() {
            let prv = &blocks[i - 1];
            let cur = &blocks[i];

            // the height of the current block is the successor of the previous block
            assert_eq!(
                cur.signed_header.header.height.value(),
                prv.signed_header.header.height.value() + 1
            );

            // the last_block_id hash is equal to the previous block's hash
            assert_eq!(
                cur.signed_header.header.last_block_id.map(|lbi| lbi.hash),
                Some(prv.signed_header.header.hash())
            );
        }
    }
}
