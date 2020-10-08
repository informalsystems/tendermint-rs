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
        let mut light_blocks = Vec::new();

        for _i in 2..num {
            // add "next" light block to the vector
            light_blocks.push(testgen_light_block.next());
        }

        let info = Info {
            id: light_blocks[0]
                .header
                .as_ref()
                .unwrap()
                .chain_id
                .as_ref()
                .expect("missing chain id")
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

    pub fn advance_chain(&mut self) -> Self {
        let new_light_block = self
            .light_blocks
            .last()
            .expect("Cannot find testgen light block")
            .next();
        let advanced_light_blocks = &mut self.light_blocks;
        advanced_light_blocks.push(new_light_block);

        let height = self.info.height.value() + 1;

        let info = Info {
            id: self.info.id,
            height: Height::from(height),
            // TODO: figure how to add this
            last_block_id: None,
            // TODO: Not sure yet what this time means
            time: None,
        };

        Self::new(info, advanced_light_blocks.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tendermint::block::Height;

    #[test]
    fn test_advance_chain() {
        let vals = Validator::new("val-1");
        let light_blocks = vec![LightBlock::new_default(&[vals], 1)];
        let info = Info {
            id: light_blocks[0]
                .header
                .as_ref()
                .unwrap()
                .chain_id
                .as_ref()
                .expect("missing chain id")
                .parse()
                .unwrap(),
            height: Height::from(1 as u32),
            last_block_id: None,
            time: None,
        };
        let advanced_light_chain = LightChain::new(info, light_blocks).advance_chain();

        assert_eq!(2, advanced_light_chain.info.height.value());
    }
}
