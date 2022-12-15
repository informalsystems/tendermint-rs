//! Tendermint Light Client JavaScript/WASM interface.
//!
//! This crate exposes some of the [`tendermint-light-client-verifier`] crate's
//! functionality to be used from the JavaScript ecosystem.
//!
//! For a detailed example, please see the [`verifier-web` example] in the
//! repository.
//!
//! [`tendermint-light-client-verifier`]: https://github.com/informalsystems/tendermint-rs/tree/main/light-client-verifier
//! [`verifier-web` example]: https://github.com/informalsystems/tendermint-rs/tree/main/light-client-js/examples/verifier-web

mod utils;

use std::time::Duration;

use serde::{Deserialize, Serialize};
use tendermint::Time;
// TODO: Use Web Crypto API for cryptographic routines.
// https://github.com/informalsystems/tendermint-rs/issues/1241
use tendermint_light_client_verifier::ProdVerifier;
use tendermint_light_client_verifier::{
    options::Options,
    types::{LightBlock, TrustThreshold},
    Verifier,
};
use wasm_bindgen::{prelude::*, JsValue};

/// Check whether a given untrusted block can be trusted.
#[wasm_bindgen]
pub fn verify(untrusted: JsValue, trusted: JsValue, options: JsValue, now: JsValue) -> JsValue {
    let result = deserialize_params(untrusted, trusted, options, now).map(
        |(untrusted, trusted, options, now)| {
            let verifier = ProdVerifier::default();
            verifier.verify(
                untrusted.as_untrusted_state(),
                trusted.as_trusted_state(),
                &options,
                now,
            )
        },
    );
    serde_wasm_bindgen::to_value(&result).unwrap()
}

fn deserialize_params(
    untrusted: JsValue,
    trusted: JsValue,
    options: JsValue,
    now: JsValue,
) -> Result<(LightBlock, LightBlock, Options, Time), Error> {
    let untrusted =
        serde_wasm_bindgen::from_value(untrusted).map_err(|e| Error::Serialization {
            param: "untrusted".into(),
            msg: e.to_string(),
        })?;

    let trusted = serde_wasm_bindgen::from_value(trusted).map_err(|e| Error::Serialization {
        param: "trusted".into(),
        msg: e.to_string(),
    })?;

    let options = serde_wasm_bindgen::from_value::<JsOptions>(options)
        .map(Into::into)
        .map_err(|e| Error::Serialization {
            param: "options".into(),
            msg: e.to_string(),
        })?;

    let now = serde_wasm_bindgen::from_value(now).map_err(|e| Error::Serialization {
        param: "now".into(),
        msg: e.to_string(),
    })?;

    Ok((untrusted, trusted, options, now))
}

/// Errors produced by this crate.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Error {
    /// A serialization/deserialization error occurred.
    #[serde(rename = "serialization")]
    Serialization { param: String, msg: String },
}

// Simplified options supplied from JavaScript.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsOptions {
    pub trust_threshold: (u64, u64),
    pub trusting_period: u64,
    pub clock_drift: u64,
}

impl From<JsOptions> for Options {
    fn from(o: JsOptions) -> Self {
        let (num, den) = o.trust_threshold;
        Self {
            trust_threshold: TrustThreshold::new(num, den).unwrap(),
            trusting_period: Duration::from_secs(o.trusting_period),
            clock_drift: Duration::from_secs(o.clock_drift),
        }
    }
}
