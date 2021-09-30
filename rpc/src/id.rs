//! JSON-RPC IDs

use crate::prelude::*;
use crate::utils::uuid_str;
use core::fmt;
use serde::{Deserialize, Serialize};

/// JSON-RPC ID: request-specific identifier
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(untagged)]
pub enum Id {
    /// Numerical JSON ID
    Num(i64),
    /// String JSON ID
    Str(String),
    /// null JSON ID
    None,
}

impl Id {
    /// Create a JSON-RPC ID containing a UUID v4 (i.e. random)
    pub fn uuid_v4() -> Self {
        Self::Str(uuid_str())
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Id::Num(i) => write!(f, "{}", i),
            Id::Str(s) => write!(f, "{}", s),
            Id::None => write!(f, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::Debug;
    use serde::{de::DeserializeOwned, Serialize};

    use super::*;

    #[test]
    fn round_tripping_jsonrpc_id() {
        let str = r#""42""#;
        serialization_roundtrip::<Id>(str);

        let str2 = r#""936DA01F-9ABD-4D9D-80C7-02AF85C822A8""#;
        serialization_roundtrip::<Id>(str2);

        let num = r#"42"#;
        serialization_roundtrip::<Id>(num);

        let zero = r#"0"#;
        serialization_roundtrip::<Id>(zero);

        let null = r#"null"#;
        serialization_roundtrip::<Id>(null);
    }

    fn serialization_roundtrip<T>(json_data: &str)
    where
        T: Debug + PartialEq + Serialize + DeserializeOwned,
    {
        let parsed0 = serde_json::from_str::<T>(json_data);
        assert!(parsed0.is_ok());
        let parsed0 = parsed0.unwrap();

        let serialized = serde_json::to_string(&parsed0);
        assert!(serialized.is_ok());
        let serialized = serialized.unwrap();

        let parsed1 = serde_json::from_str::<T>(&serialized);
        assert!(parsed1.is_ok());
        let parsed1 = parsed1.unwrap();

        assert_eq!(parsed0, parsed1);
    }
}
