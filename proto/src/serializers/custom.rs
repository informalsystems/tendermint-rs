//! Custom serializers

use serde::{Deserialize, Deserializer};

/// Parse null as default
pub fn null_as_default<'de, D, T: Default + Deserialize<'de>>(
    deserializer: D,
) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(<Option<T>>::deserialize(deserializer)?.unwrap_or_default())
}
