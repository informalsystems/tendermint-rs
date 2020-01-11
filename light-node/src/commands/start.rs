//! `start` subcommand - example of how to write a subcommand

/// App-local prelude includes `app_reader()`/`app_writer()`/`app_config()`
/// accessors along with logging macros. Customize as you see fit.
use crate::prelude::*;

use core::future::Future;
use tendermint::hash;
use tendermint::lite;
use tendermint::lite::{Error, Header, Requester, SignedHeader, Store, TrustedState};
use tendermint::rpc;
use tendermint::{block::Height, Hash};
use tokio::runtime::Builder;

use tendermint::lite::ValidatorSet as _;

use crate::config::LightNodeConfig;
use crate::requester::RPCRequester;
use crate::state::State;
use crate::store::MemStore;
use crate::threshold::TrustThresholdOneThird;

use abscissa_core::{config, Command, FrameworkError, Options, Runnable};
use std::time::{Duration, SystemTime};

/// `start` subcommand
///
/// The `Options` proc macro generates an option parser based on the struct
/// definition, and is defined in the `gumdrop` crate. See their documentation
/// for a more comprehensive example:
///
/// <https://docs.rs/gumdrop/>
#[derive(Command, Debug, Options)]
pub struct StartCmd {
    /// RPC address to request headers and validators from.
    #[options(free)]
    rpc_addr: String,
}

// TODO: this should also somehow be configurable ...
// we can't simply add this as a field in the config because this either would
// be a trait (`TrustThreshold`) or immediately and impl thereof (`TrustThresholdOneThird`).
static THRESHOLD: &TrustThresholdOneThird = &TrustThresholdOneThird {};

impl Runnable for StartCmd {
    /// Start the application.
    fn run(&self) {
        let config = app_config();

        let client = block_on(rpc::Client::new(&config.rpc_address.parse().unwrap())).unwrap();
        let req = RPCRequester::new(client);
        let mut store = MemStore::new();

        let vals_hash = Hash::from_hex_upper(
            hash::Algorithm::Sha256,
            &config.subjective_init.validators_hash,
        )
        .unwrap();

        println!("Requesting from {}.", config.rpc_address);

        subjective_init(
            Height::from(config.subjective_init.height),
            vals_hash,
            &mut store,
            &req,
        )
        .unwrap();

        loop {
            let latest = (&req).signed_header(0).unwrap();
            let latest_peer_height = latest.header().height();

            let latest = store.get(Height::from(0)).unwrap();
            let latest_height = latest.last_header().header().height();

            // only bisect to higher heights
            if latest_peer_height <= latest_height {
                std::thread::sleep(Duration::new(1, 0));
                continue;
            }

            println!(
                "attempting bisection from height {:?} to height {:?}",
                store
                    .get(Height::from(0))
                    .unwrap()
                    .last_header()
                    .header()
                    .height(),
                latest_peer_height,
            );

            let now = &SystemTime::now();
            lite::verify_and_update_bisection(
                latest_peer_height,
                THRESHOLD, // TODO
                &config.trusting_period,
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
}

impl config::Override<LightNodeConfig> for StartCmd {
    // Process the given command line options, overriding settings from
    // a configuration file using explicit flags taken from command-line
    // arguments.
    fn override_config(
        &self,
        mut config: LightNodeConfig,
    ) -> Result<LightNodeConfig, FrameworkError> {
        if !self.rpc_addr.is_empty() {
            config.rpc_address = self.rpc_addr.to_owned();
        }

        Ok(config)
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
fn subjective_init(
    height: Height,
    vals_hash: Hash,
    store: &mut MemStore,
    req: &RPCRequester,
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

    let signed_header = req.signed_header(height)?;

    // TODO: validate signed_header.commit() with the vals ...

    let next_vals = req.validator_set(height.increment())?;

    // TODO: check next_vals ...

    let trusted_state = &State::new(&signed_header, &next_vals);

    store.add(trusted_state)?;

    Ok(())
}

fn block_on<F: Future>(future: F) -> F::Output {
    Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}
