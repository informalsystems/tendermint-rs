use crate::light_block::LightBlock;
use crate::Validator;
use tendermint::block::Height;
use tendermint::chain::Info;

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
        let vals = Validator::new("val-1");
        let testgen_light_block = LightBlock::new_default(&[vals], 1);
        let mut light_blocks: Vec<LightBlock> = vec![testgen_light_block.clone()];

        for _i in 2..num {
            // add "next" light block to the vector
            light_blocks.push(testgen_light_block.next());
        }

        let info = Info {
            id: light_blocks[0]
                .chain_id()
                .parse()
                .unwrap(),
            height: Height::from(num),
            // TODO: figure how to add this
            last_block_id: None,
            // TODO: Not sure yet what this time means
            time: None,
        };
        Self::new(info, light_blocks)
    }

    /// expects at least one LightBlock in the Chain
    pub fn advance_chain(&mut self) -> LightBlock {
        let last_light_block = self
            .light_blocks
            .last()
            .expect("Cannot find testgen light block");

        let new_light_block = last_light_block.next();
        self.light_blocks.push(new_light_block.clone());

        self.info.height = Height(new_light_block.height());

        new_light_block
    }

    /// fetches a block from LightChain at a certain height
    /// it returns None if a block does not exist for the target_height
    pub fn block(&self, target_height: u64) -> Option<LightBlock> {
        self.light_blocks
            .clone()
            .into_iter()
            .find(|lb| lb.height() == target_height)
    }

    /// fetches the latest block from LightChain
    pub fn latest_block(&self) -> LightBlock {
        self.light_blocks
            .last()
            .expect("cannot find last light block")
            .clone()
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
        assert_eq!(2, light_chain.info.height.0);

        let advance_2 = light_chain.advance_chain();

        assert_eq!(3, advance_2.height());
        assert_eq!(3, light_chain.info.height.0);
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
}
