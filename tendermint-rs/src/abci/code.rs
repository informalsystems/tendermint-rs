use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

/// ABCI application response codes.
///
/// These presently use 0 for success and non-zero for errors:
///
/// <https://tendermint.com/docs/spec/abci/abci.html#errors>
///
/// Note that in the future there may potentially be non-zero success codes.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Code {
    /// Success
    Ok,

    /// Error codes
    Err(u32),
}

impl Code {
    /// Was the response OK?
    pub fn is_ok(self) -> bool {
        match self {
            Code::Ok => true,
            Code::Err(_) => false,
        }
    }

    /// Was the response an error?
    pub fn is_err(self) -> bool {
        !self.is_ok()
    }

    /// Get the integer error value for this code
    pub fn value(self) -> u32 {
        u32::from(self)
    }
}

impl From<u32> for Code {
    fn from(value: u32) -> Code {
        match value {
            0 => Code::Ok,
            err => Code::Err(err),
        }
    }
}

impl From<Code> for u32 {
    fn from(code: Code) -> u32 {
        match code {
            Code::Ok => 0,
            Code::Err(err) => err,
        }
    }
}

impl<'de> Deserialize<'de> for Code {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Code::from(
            String::deserialize(deserializer)?
                .parse::<u32>()
                .map_err(|e| D::Error::custom(format!("{}", e)))?,
        ))
    }
}

impl Serialize for Code {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value().serialize(serializer)
    }
}
