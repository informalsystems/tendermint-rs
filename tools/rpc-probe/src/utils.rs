//! Utility methods.

use std::path::Path;

use getrandom::getrandom;
use subtle_encoding::{base64, hex};

use crate::error::Result;

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

pub async fn write_json(base_path: &Path, name: &str, v: &serde_json::Value) -> Result<()> {
    let path = base_path.join(format!("{}.json", name));
    tokio::fs::write(path, serde_json::to_string_pretty(v).unwrap()).await?;
    Ok(())
}

/// Sanitizes the given string such that it's acceptable to be used as a
/// filename.
pub fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => c,
            _ => '_',
        })
        .fold(String::new(), |mut a, b| {
            a.push(b);
            a
        })
}
