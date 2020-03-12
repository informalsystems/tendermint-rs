use tendermint::hash;
use tendermint::lite;
use tendermint::lite::TrustThresholdFraction;
use tendermint::lite::{Header as _, Requester as _, ValidatorSet as _};
use tendermint::rpc;
use tendermint::{block::Height, Hash};

use tendermint_lite::{requester::RPCRequester, store::MemStore};

use std::time::{Duration, SystemTime};
use tendermint::lite::error::Error;
use tendermint_lite::store::State;

// TODO: these should be config/args
static SUBJECTIVE_HEIGHT: u64 = 1;
static SUBJECTIVE_VALS_HASH_HEX: &str =
    "A5A7DEA707ADE6156F8A981777CA093F178FC790475F6EC659B6617E704871DD";
static RPC_ADDR: &str = "localhost:26657";

#[tokio::main]
async fn main() {
    // TODO: this should be config
    let trusting_period = Duration::new(6000, 0);

    // setup requester for primary peer
    let client = rpc::Client::new(RPC_ADDR.parse().unwrap());

    if let Err(err) = client.health().await {
        eprintln!("error: health check failed: {}", err);
        std::process::exit(1);
    }

    let req = RPCRequester::new(client);
    let mut store = MemStore::new();

    let vals_hash =
        Hash::from_hex_upper(hash::Algorithm::Sha256, SUBJECTIVE_VALS_HASH_HEX).unwrap();

    subjective_init(Height::from(SUBJECTIVE_HEIGHT), vals_hash, &mut store, &req)
        .await
        .unwrap();

    loop {
        let latest = (&req).signed_header(0).await.unwrap();
        let latest_peer_height = latest.header().height();

        let latest = store.get(0).unwrap();
        let latest_height = latest.last_header().header().height();

        // only bisect to higher heights
        if latest_peer_height <= latest_height {
            std::thread::sleep(Duration::new(1, 0));
            continue;
        }

        println!(
            "attempting bisection from height {:?} to height {:?}",
            store.get(0).unwrap().last_header().header().height(),
            latest_peer_height,
        );

        let now = SystemTime::now();
        let trusted_state = store.get(0).expect("can not read trusted state");

        let new_states = lite::verify_bisection(
            trusted_state.clone(),
            latest_peer_height,
            TrustThresholdFraction::default(),
            trusting_period,
            now,
            &req,
        )
        .await
        .unwrap();

        for new_state in new_states {
            store
                .add(new_state)
                .expect("couldn't store new trusted state");
        }

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
 * TODO: this should take traits ... but how to deal with the State ?
 * TODO: better name ?
 */
async fn subjective_init(
    height: Height,
    vals_hash: Hash,
    store: &mut MemStore,
    req: &RPCRequester,
) -> Result<(), Error> {
    if store.get(height.value()).is_ok() {
        // we already have this !
        return Ok(());
    }

    // check that the val hash matches
    let vals = req.validator_set(height.value()).await?;

    if vals.hash() != vals_hash {
        // TODO
        panic!("vals hash dont match")
    }

    let signed_header = req.signed_header(SUBJECTIVE_HEIGHT).await?;

    // TODO: validate signed_header.commit() with the vals ...

    let next_vals = req.validator_set(height.increment().value()).await?;

    // TODO: check next_vals ...

    let trusted_state = State::new(signed_header, next_vals);

    store.add(trusted_state)?;

    Ok(())
}
