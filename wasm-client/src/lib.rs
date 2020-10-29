mod utils;

use std::time::Duration;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use tendermint::Time;
use tendermint_light_client::{
    components::verifier::ProdVerifier, components::verifier::Verdict,
    components::verifier::Verifier, light_client::Options as LightOptions, types::LightBlock,
    types::TrustThreshold,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    Serde(String),
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn verify(untrusted: &JsValue, trusted: &JsValue, options: &JsValue, now: &JsValue) -> JsValue {
    let result = verify_inner(untrusted, trusted, options, now);
    JsValue::from_serde(&result).unwrap()
}

#[derive(Debug, Serialize, Deserialize)]
struct Options {
    pub trust_threshold: (u64, u64),
    pub trusting_period: u64,
    pub clock_drift: u64,
}

impl From<Options> for LightOptions {
    fn from(o: Options) -> Self {
        let (num, dem) = o.trust_threshold;

        Self {
            trust_threshold: TrustThreshold::new(num, dem).unwrap(),
            trusting_period: Duration::from_secs(o.trusting_period),
            clock_drift: Duration::from_secs(o.clock_drift),
        }
    }
}

#[inline]
fn verify_inner(
    untrusted: &JsValue,
    trusted: &JsValue,
    options: &JsValue,
    now: &JsValue,
) -> Result<Verdict, Error> {
    let untrusted: LightBlock = untrusted
        .into_serde()
        .map_err(|e| Error::Serde(e.to_string()))?;

    let trusted: LightBlock = trusted
        .into_serde()
        .map_err(|e| Error::Serde(e.to_string()))?;

    let options: Options = options
        .into_serde()
        .map_err(|e| Error::Serde(e.to_string()))?;

    let light_options = options.into();

    let now: Time = now.into_serde().map_err(|e| Error::Serde(e.to_string()))?;

    let verifier = ProdVerifier::default();
    let verdict = verifier.verify(&untrusted, &trusted, &light_options, now);

    Ok(verdict)
}
