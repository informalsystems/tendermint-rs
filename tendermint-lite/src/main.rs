use tendermint::{block::Height, Hash};
use tendermint::hash::Algorithm;
use tendermint::net;
use tendermint::rpc;
use tendermint::lite;
use tendermint::lite::Error;
use tendermint::lite::{Requester as _, Store as _, TrustedState as _, SignedHeader as _, ValidatorSet as _, Header as _, TrustThreshold};

use tendermint_lite::{requester::Requester, store::MemStore, state::State, threshold::TrustThresholdOneThird};

use core::future::Future;
use tokio::runtime::Builder;
use std::time::{Duration, SystemTime};

// TODO: these should be config/args
static subjective_height: u64 = 1;
static subjective_vals_hash_hex: &str= "A5A7DEA707ADE6156F8A981777CA093F178FC790475F6EC659B6617E704871DD";
static rpc_addr: &str= "localhost:26657";

// TODO: this should somehow be configurable ...
static threshold: &TrustThresholdOneThird = &TrustThresholdOneThird{};

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
    let client = block_on(rpc::Client::new(&rpc_addr.parse().unwrap())).unwrap();
    let req = Requester::new(client);
    let mut store = MemStore::new();

    let vals_hash = Hash::from_hex_upper(Algorithm::Sha256, subjective_vals_hash_hex).unwrap();


    validate_and_init_subjective_state(
        Height::from(subjective_height),
        vals_hash,
        &mut store,
        &req,
    ).unwrap();

    loop {
        // NOTE: 0 is a bad idea. use an Enum{ Height, LatestHeight } or something
        // instead ..
        let latest = (&req).signed_header(0).unwrap();
        let latest_height = latest.header().height();

	println!(
	    "attempting bisection from height {:?} to height {:?}",
	    store.get(Height::from(0)).unwrap().last_header().header().height(),
	    latest_height,
	);

        let now = &SystemTime::now();
        lite::verify_and_update_bisection(
            latest_height,
            threshold,
            &trusting_period, 
            now,
            &req,
            &mut store,
        ).unwrap();

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
    req: &Requester
) -> Result<(), Error> {


    if let Ok(_) = store.get(height) {
        // we already have this !
        return Ok(())
    }

    // check that the val hash matches
    let vals = req.validator_set(height)?;

    if vals.hash() != vals_hash {
        // TODO
        panic!("vals hash dont match")
    }

    let signed_header = req.signed_header(subjective_height)?;

    // TODO: validate signed_header.commit() with the vals ...
    
    let next_vals = req.validator_set(height.increment())?;

    // TODO: check next_vals ...
    
    let trusted_state = &State::new(&signed_header, &next_vals);

    store.add(trusted_state)?;

    Ok(())
}
