use std::{collections::HashMap, time::Duration};

use tendermint::{hash::Algorithm, Hash, Time};

use tendermint_light_client::{
    components::{
        io::{AtHeight, Io},
        scheduler,
        verifier::ProdVerifier,
    },
    errors::Error,
    light_client::{LightClient, Options},
    operations::ProdHasher,
    state::State,
    store::{memory::MemoryStore, LightStore},
    tests::{MockClock, MockIo},
    types::{Height, LightBlock, Status},
};

use tendermint_testgen::{
    light_block::{default_peer_id, TMLightBlock as TGLightBlock},
    Generator, LightChain,
};

use proptest::{prelude::*, test_runner::TestRng};

fn testgen_to_lb(tm_lb: TGLightBlock) -> LightBlock {
    LightBlock {
        signed_header: tm_lb.signed_header,
        validators: tm_lb.validators,
        next_validators: tm_lb.next_validators,
        provider: tm_lb.provider,
    }
}

#[derive(Clone, Debug)]
struct TestCase {
    length: u32,
    chain: LightChain,
    target_height: Height,
    trusted_height: Height,
}

fn make(chain: LightChain, trusted_height: Height) -> (LightClient, State) {
    let primary = default_peer_id();
    let chain_id = "testchain-1".parse().unwrap();

    let clock = MockClock { now: Time::now() };

    let options = Options {
        trust_threshold: Default::default(),
        trusting_period: Duration::from_secs(60 * 60 * 24 * 10),
        clock_drift: Duration::from_secs(10),
    };

    let light_blocks = chain
        .light_blocks
        .into_iter()
        .map(|lb| lb.generate().unwrap())
        .map(testgen_to_lb)
        .collect();

    let io = MockIo::new(chain_id, light_blocks);

    let trusted_state = io
        .fetch_light_block(AtHeight::At(trusted_height))
        .expect("could not find trusted light block");

    let mut light_store = MemoryStore::new();
    light_store.insert(trusted_state, Status::Trusted);

    let state = State {
        light_store: Box::new(light_store),
        verification_trace: HashMap::new(),
    };

    let verifier = ProdVerifier::default();
    let hasher = ProdHasher::default();

    let light_client = LightClient::new(
        primary,
        options,
        clock,
        scheduler::basic_bisecting_schedule,
        verifier,
        hasher,
        io,
    );

    (light_client, state)
}

fn verify(tc: TestCase) -> Result<LightBlock, Error> {
    let (light_client, mut state) = make(tc.chain, tc.trusted_height);
    light_client.verify_to_target(tc.target_height, &mut state)
}

fn ok_test(tc: TestCase) -> Result<(), TestCaseError> {
    let target_height = tc.target_height;
    let result = verify(tc);

    prop_assert!(result.is_ok());
    prop_assert_eq!(result.unwrap().height(), target_height);

    Ok(())
}

fn bad_test(tc: TestCase) -> Result<(), TestCaseError> {
    let result = verify(tc);
    prop_assert!(result.is_err());
    Ok(())
}

fn testcase(max: u32) -> impl Strategy<Value = TestCase> {
    (1..=max).prop_flat_map(move |length| {
        (1..=length).prop_flat_map(move |trusted_height| {
            (1..=trusted_height).prop_map(move |target_height| TestCase {
                chain: LightChain::default_with_length(length as u64),
                length,
                trusted_height: trusted_height.into(),
                target_height: target_height.into(),
            })
        })
    })
}

fn remove_last_block_id_hash(mut tc: TestCase, mut rng: TestRng) -> TestCase {
    let from = tc.target_height.value() + 1;
    let to = tc.trusted_height.value() + 1;
    let height = rng.gen_range(from, to);

    dbg!(tc.target_height, tc.trusted_height, height);

    let block = tc.chain.block_mut(height).unwrap();

    if let Some(header) = block.header.as_mut() {
        header.last_block_id_hash = None;
    }

    tc
}

fn corrupt_hash(mut tc: TestCase, mut rng: TestRng) -> TestCase {
    let from = tc.target_height.value();
    let to = tc.trusted_height.value();
    let height = rng.gen_range(from, to);

    dbg!(tc.target_height, tc.trusted_height, height);

    let block = tc.chain.block_mut(height).unwrap();

    if let Some(header) = block.header.as_mut() {
        header.time = Some(1610105021);
    }

    tc
}

fn corrupt_last_block_id_hash(mut tc: TestCase, mut rng: TestRng) -> TestCase {
    let from = tc.target_height.value() + 1;
    let to = tc.trusted_height.value() + 1;
    let height = rng.gen_range(from, to);

    dbg!(tc.target_height, tc.trusted_height, height);

    let block = tc.chain.block_mut(height).unwrap();

    if let Some(header) = block.header.as_mut() {
        let hash = Hash::from_hex_upper(
            Algorithm::Sha256,
            "C68B4CFC7F9AA239F9E0DF7CDEF264DD1CDFE8B73EF04B5600A20111144F42BF",
        )
        .unwrap();

        header.last_block_id_hash = Some(hash);
    }

    tc
}

fn tc_missing_last_block_id_hash(max: u32) -> impl Strategy<Value = TestCase> {
    testcase(max)
        .prop_filter("target == trusted", |tc| {
            tc.target_height != tc.trusted_height
        })
        .prop_perturb(remove_last_block_id_hash)
}

fn tc_corrupted_last_block_id_hash(max: u32) -> impl Strategy<Value = TestCase> {
    testcase(max)
        .prop_filter("target == trusted", |tc| {
            tc.target_height != tc.trusted_height
        })
        .prop_perturb(corrupt_last_block_id_hash)
}

fn tc_corrupted_hash(max: u32) -> impl Strategy<Value = TestCase> {
    testcase(max)
        .prop_filter("target == trusted", |tc| {
            tc.target_height != tc.trusted_height
        })
        .prop_perturb(corrupt_hash)
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 20,
        max_shrink_iters: 0,
        ..Default::default()
    })]

    #[test]
    fn prop_target_equal_trusted_first_block(mut tc in testcase(100)) {
        tc.target_height = 1_u32.into();
        tc.trusted_height = 1_u32.into();
        ok_test(tc)?;
    }

    #[test]
    fn prop_target_equal_trusted_last_block(mut tc in testcase(100)) {
        tc.target_height = tc.length.into();
        tc.trusted_height = tc.length.into();
        ok_test(tc)?;
    }

    #[test]
    fn prop_target_equal_trusted(mut tc in testcase(100)) {
        tc.target_height = tc.trusted_height;
        ok_test(tc)?;
    }

    #[test]
    fn prop_two_ends(mut tc in testcase(100)) {
        tc.target_height = 1_u32.into();
        tc.trusted_height = tc.length.into();
        ok_test(tc)?;
    }

    #[test]
    fn prop_target_less_than_trusted(tc in testcase(100)) {
        ok_test(tc)?;
    }

    #[test]
    fn missing_last_block_id_hash(tc in tc_missing_last_block_id_hash(100)) {
        bad_test(tc)?;
    }

    #[test]
    fn corrupted_last_block_id_hash(tc in tc_corrupted_last_block_id_hash(100)) {
        bad_test(tc)?;
    }

    #[test]
    fn corrupted_hash(tc in tc_corrupted_hash(100)) {
        bad_test(tc)?;
    }
}
