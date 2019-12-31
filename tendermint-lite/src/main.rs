use tendermint::hash;
use tendermint::lite;
use tendermint::lite::Error;
use tendermint::lite::{
    Header as _, Requester as _, SignedHeader as _, Store as _, TrustedState as _,
    ValidatorSet as _,
};
use tendermint::rpc;
use tendermint::{block::Height, Hash};

use tendermint_lite::{
    requester::Requester, state::State, store::MemStore, threshold::TrustThresholdOneThird,
};

use core::future::Future;
use std::time::{Duration, SystemTime};
use tokio::runtime::Builder;

// TODO: these should be config/args
static SUBJECTIVE_HEIGHT: u64 = 1;
static SUBJECTIVE_VALS_HASH_HEX: &str =
    "A5A7DEA707ADE6156F8A981777CA093F178FC790475F6EC659B6617E704871DD";
static RPC_ADDR: &str = "localhost:26657";

// TODO: this should somehow be configurable ...
static THRESHOLD: &TrustThresholdOneThird = &TrustThresholdOneThird {};

pub fn block_on<F: Future>(future: F) -> F::Output {
    Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}

fn main() {
    // TODO: this should be config
    let trusting_period = Duration::new(600, 0);

    // setup requester for primary peer
    let client = block_on(rpc::Client::new(&RPC_ADDR.parse().unwrap())).unwrap();
    let req = Requester::new(client);
    let mut store = MemStore::new();

    let vals_hash =
        Hash::from_hex_upper(hash::Algorithm::Sha256, SUBJECTIVE_VALS_HASH_HEX).unwrap();

    validate_and_init_subjective_state(
        Height::from(SUBJECTIVE_HEIGHT),
        vals_hash,
        &mut store,
        &req,
    )
    .unwrap();

    loop {
        // NOTE: 0 is a bad idea. use an Enum{ Height, LatestHeight } or something
        // instead ..
        let latest = (&req).signed_header(0).unwrap();
        let latest_height = latest.header().height();

        println!(
            "attempting bisection from height {:?} to height {:?}",
            store
                .get(Height::from(0))
                .unwrap()
                .last_header()
                .header()
                .height(),
            latest_height,
        );

        let now = &SystemTime::now();
        lite::verify_and_update_bisection(
            latest_height,
            THRESHOLD,
            &trusting_period,
            now,
            &req,
            &mut store,
        )
        .unwrap();

        println!("Succeeded bisecting!");

        // notifications ?

        // sleep for a few secs ?
    }
}

/*
 * The following is initialization logic that should have a
 * function in the lite crate like:
 * `subjective_init(height, vals_hash, store, requester) -> Result<(), Error`
 * it would fetch the initial header/vals from the requester and populate a
 * trusted state and store it in the store ...
 * TODO: this should take traits ...
 * TODO: better name ?
*/
fn validate_and_init_subjective_state(
    height: Height,
    vals_hash: Hash,
    store: &mut MemStore,
    req: &Requester,
) -> Result<(), Error> {
    if store.get(height).is_ok() {
        // we already have this !
        return Ok(());
    }

    // check that the val hash matches
    let vals = req.validator_set(height)?;

    if vals.hash() != vals_hash {
        // TODO
        panic!("vals hash dont match")
    }

    let signed_header = req.signed_header(SUBJECTIVE_HEIGHT)?;

    // TODO: validate signed_header.commit() with the vals ...

    let next_vals = req.validator_set(height.increment())?;

    // TODO: check next_vals ...

    let trusted_state = &State::new(&signed_header, &next_vals);

    store.add(trusted_state)?;

    Ok(())
}
