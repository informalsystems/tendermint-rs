use tendermint::Hash;
use tendermint_light_client::types::{Height, LightBlock};
use tendermint_testgen as tg;
use tg::Generator;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Block {
    height: Height,
    hash: Hash,
    last_block_id_hash: Option<Hash>,
}

#[test]
fn light_chain() {
    let chain = tg::LightChain::default_with_length(3);
    dbg!(chain.info);

    let blocks = chain
        .light_blocks
        .into_iter()
        .map(|lb| lb.generate().unwrap())
        .map(testgen_to_lb)
        .map(|lb| {
            let hash = lb.signed_header.header.hash();
            let last_block_id_hash = lb.signed_header.header.last_block_id.map(|id| id.hash);

            Block {
                height: lb.height(),
                hash,
                last_block_id_hash,
            }
        })
        .collect::<Vec<_>>();

    assert!(blocks[0].last_block_id_hash.is_none());

    for i in 1..blocks.len() {
        let prv = blocks[i - 1];
        let cur = blocks[i];

        assert_eq!(cur.height.value(), prv.height.value() + 1);
        assert_eq!(cur.last_block_id_hash, Some(prv.hash));
    }
}

fn testgen_to_lb(lb: tg::light_block::TMLightBlock) -> LightBlock {
    LightBlock {
        signed_header: lb.signed_header,
        validators: lb.validators,
        next_validators: lb.next_validators,
        provider: lb.provider,
    }
}
