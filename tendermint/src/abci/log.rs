use crate::{Error, Kind};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use tendermint_proto::serializers::log::Log as RawLog;

/// ABCI log data
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "RawLog", into = "RawLog")]
pub struct Log(serde_json::Value);

impl TryFrom<serde_json::Value> for Log {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        if !(value.is_array() || value.is_string() || value.is_null()) {
            return Err(Kind::InvalidLog
                .context("log must be array, string or empty")
                .into());
        }
        Ok(Self(value))
    }
}

impl From<Log> for serde_json::Value {
    fn from(value: Log) -> Self {
        value.0
    }
}

impl TryFrom<RawLog> for Log {
    type Error = Error;

    fn try_from(value: RawLog) -> Result<Self, Self::Error> {
        value.0.try_into()
    }
}

impl From<Log> for RawLog {
    fn from(value: Log) -> Self {
        RawLog(value.into())
    }
}

impl FromStr for Log {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(serde_json::Value::String(s.to_string()))
    }
}

impl Log {
    /// Convenience function: get value
    pub fn value(&self) -> &serde_json::Value {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::abci::Log;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::convert::TryFrom;

    #[test]
    fn array_of_objects() {
        #[derive(Debug, Deserialize, Serialize, PartialEq)]
        struct L {
            log: Log,
        }
        let json = r#"{"log":"[{\"events\":[{\"type\":\"connection_open_init\",\"attributes\":[{\"key\":\"connection_id\",\"value\":\"ibc0to1_cid55\"},{\"key\":\"client_id\",\"value\":\"ibconeclient\"},{\"key\":\"counterparty_client_id\",\"value\":\"ibczeroclient\"},{\"key\":\"counterparty_connection_id\"}]},{\"type\":\"message\",\"attributes\":[{\"key\":\"action\",\"value\":\"connection_open_init\"},{\"key\":\"module\",\"value\":\"ibc_connection\"}]}]}]"}"#;
        let mystruct: L = serde_json::from_str(json).unwrap();
        assert!(mystruct.log.value().is_array());
        assert!(mystruct.log.value().get(0).unwrap().is_object());
        assert_eq!(
            mystruct
                .log
                .value()
                .get(0)
                .unwrap()
                .as_object()
                .unwrap()
                .get("events")
                .unwrap()
                .get(0)
                .unwrap()
                .get("type")
                .unwrap()
                .as_str()
                .unwrap(),
            "connection_open_init"
        );
        let mystring = serde_json::to_string(&mystruct).unwrap();
        // We can't compare the JSON strings directly, as they might be in different order.
        let mystruct2: L = serde_json::from_str(&mystring).unwrap();
        assert_eq!(mystruct, mystruct2);
    }

    #[test]
    fn test_simple_struct_serialization() {
        let log = Log::try_from(json!([{
            "code": 200,
            "success": true,
            "payload": {
                "features": [
                    "serde",
                    "json"
                ]
            }
        }]))
        .unwrap();
        let mystring: String = serde_json::to_string(&log).unwrap();
        assert_eq!(
            mystring,
            "\"[{\\\"code\\\":200,\\\"payload\\\":{\\\"features\\\":[\\\"serde\\\",\\\"json\\\"]},\\\"success\\\":true}]\""
        )
    }
}
