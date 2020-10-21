//! Serialize/deserialize Vec<Vec<u8>> type from and into transactions (HexString array).
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use subtle_encoding::hex;

/// Deserialize transactions into Vec<Vec<u8>>
pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value_vec_hexstring = Option::<Vec<String>>::deserialize(deserializer)?;
    if value_vec_hexstring.is_none() {
        return Ok(Vec::new());
    }
    let value_vec_hexstring = value_vec_hexstring.unwrap();
    if value_vec_hexstring.is_empty() {
        return Ok(Vec::new());
    }
    value_vec_hexstring
        .into_iter()
        .map(|s| {
            hex::decode_upper(&s)
                .or_else(|_| hex::decode(&s))
                .map_err(serde::de::Error::custom)
        })
        .collect()
}

/// Serialize from Vec<Vec<u8>> into transactions
pub fn serialize<S>(value: &[Vec<u8>], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if value.is_empty() {
        let whatevs: Option<Vec<u8>> = None;
        return whatevs.serialize(serializer);
    }
    let value_hexstring: Result<Vec<String>, S::Error> = value
        .iter()
        .map(|v| String::from_utf8(hex::encode_upper(v)).map_err(serde::ser::Error::custom))
        .collect();
    value_hexstring?.serialize(serializer)
}
