//! Utility methods.

use getrandom::getrandom;
use subtle_encoding::{base64, hex};

pub fn uuid_v4() -> String {
    let mut bytes = [0; 16];
    getrandom(&mut bytes).expect("RNG failure!");

    let uuid = uuid::Builder::from_bytes(bytes)
        .set_variant(uuid::Variant::RFC4122)
        .set_version(uuid::Version::Random)
        .build();

    uuid.to_string()
}

pub fn encode_kvpair(key: &str, value: &str) -> String {
    let kvpair = format!("{}={}", key, value);
    String::from_utf8(base64::encode(kvpair.as_bytes())).unwrap()
}

pub fn hex_string(s: &str) -> String {
    String::from_utf8(hex::encode(s.as_bytes())).unwrap()
}
