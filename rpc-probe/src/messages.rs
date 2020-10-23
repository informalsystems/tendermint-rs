//! Messages for interacting with the Tendermint RPC.

use getrandom::getrandom;
use serde_json::json;

pub fn request_wrapper(method: &str, params: serde_json::Value) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": uuid_v4(),
        "method": method,
        "params": params,
    })
}

fn uuid_v4() -> String {
    let mut bytes = [0; 16];
    getrandom(&mut bytes).expect("RNG failure!");

    let uuid = uuid::Builder::from_bytes(bytes)
        .set_variant(uuid::Variant::RFC4122)
        .set_version(uuid::Version::Random)
        .build();

    uuid.to_string()
}
