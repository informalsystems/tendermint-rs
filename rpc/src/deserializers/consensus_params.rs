use serde::{Deserialize, Deserializer};

/// Deserialize `null` or an empty object `{}` as `None`.
pub fn allow_empty_object<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    #[derive(Deserialize)]
    #[serde(
        untagged,
        deny_unknown_fields,
        expecting = "object, empty object or null"
    )]
    enum Helper<T> {
        Data(T),
        Empty {},
        Null,
    }

    match Helper::deserialize(deserializer) {
        Ok(Helper::Data(data)) => Ok(Some(data)),
        Ok(Helper::Empty {} | Helper::Null) => Ok(None),
        Err(e) => Err(e),
    }
}
