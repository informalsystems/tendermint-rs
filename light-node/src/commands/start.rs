//! `start` subcommand - start the light node.

/// App-local prelude includes `app_reader()`/`app_writer()`/`app_config()`
/// accessors along with logging macros. Customize as you see fit.
use crate::prelude::*;

use tendermint::hash;
use tendermint::lite;
use tendermint::lite::ValidatorSet as _;
use tendermint::lite::{Header, Height, Requester, TrustThresholdFraction};
use tendermint::rpc;
use tendermint::Hash;

use crate::application::APPLICATION;
use crate::config::LightNodeConfig;
use crate::requester::RPCRequester;
use crate::store::{MemStore, State};
use abscissa_core::{config, Command, FrameworkError, Options, Runnable};
use std::process;
use std::time::{Duration, SystemTime};
use tendermint::lite::error::Error;

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

impl Runnable for StartCmd {
    /// Start the application.
    fn run(&self) {
        if let Err(err) = abscissa_tokio::run(&APPLICATION, async {
            let config = app_config();

            let client = rpc::Client::new(config.rpc_address.parse().unwrap());
            let mut req = RPCRequester::new(client);
            let mut store = MemStore::new();

            let vals_hash = Hash::from_hex_upper(
                hash::Algorithm::Sha256,
                &config.subjective_init.validators_hash,
            )
            .unwrap();

            println!("Requesting from {}.", config.rpc_address);

            subjective_init(
                config.subjective_init.height,
                vals_hash,
                &mut store,
                &mut req,
            )
            .await
            .unwrap();

            loop {
                let latest_sh = (&mut req).signed_header(0).await.unwrap();
                let latest_peer_height = latest_sh.header().height();

                let latest_trusted = store.get(0).unwrap();
                let latest_trusted_height = latest_trusted.last_header().header().height();

                // only bisect to higher heights
                if latest_peer_height <= latest_trusted_height {
                    std::thread::sleep(Duration::new(1, 0));
                    continue;
                }

                println!(
                    "attempting bisection from height {:?} to height {:?}",
                    latest_trusted_height, latest_peer_height,
                );

                let now = SystemTime::now();
                lite::verify_bisection(
                    latest_trusted.to_owned(),
                    latest_peer_height,
                    TrustThresholdFraction::default(), // TODO
                    config.trusting_period,
                    now,
                    &mut req,
                )
                .await
                .unwrap();

                println!("Succeeded bisecting!");

                // notifications ?

                // sleep for a few secs ?
            }
        }) {
            eprintln!("Error while running application: {}", err);
            process::exit(1);
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
async fn subjective_init(
    height: Height,
    vals_hash: Hash,
    store: &mut MemStore,
    req: &mut RPCRequester,
) -> Result<(), Error> {
    if store.get(height).is_ok() {
        // we already have this !
        return Ok(());
    }

    // check that the val hash matches
    let vals = req.validator_set(height).await?;

    if vals.hash() != vals_hash {
        // TODO
        panic!("vals hash dont match")
    }

    let signed_header = req.signed_header(height).await?;

    // TODO: validate signed_header.commit() with the vals ...

    let next_vals = req.validator_set(height + 1).await?;

    // TODO: check next_vals ...

    let trusted_state = &State::new(signed_header, next_vals);

    store.add(trusted_state.to_owned())?;

    Ok(())
}
