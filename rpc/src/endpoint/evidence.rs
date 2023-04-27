//! `/broadcast_evidence`: broadcast an evidence.

use serde::{ser::SerializeStruct, Deserialize, Serialize};
use tendermint::{evidence::Evidence, Hash};

use crate::{client::CompatMode, dialect::Dialect, request::RequestMessage, Method};

/// `/broadcast_evidence`: broadcast an evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    /// Evidence to broadcast
    pub evidence: Evidence,

    pub compat_mode: CompatMode,
}

// This is a workaround for the fact that the serialization of the evidence
// changed between Tendermint 0.34 and 0.37.
impl Serialize for Request {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.compat_mode {
            CompatMode::V0_34 => {
                use tendermint_proto::v0_34::types::Evidence as RawEvidence;
                let mut state = serializer.serialize_struct("Request", 1)?;
                state.serialize_field("evidence", &RawEvidence::from(self.evidence.clone()))?;
                state.end()
            },
            CompatMode::V0_37 => {
                use tendermint_proto::v0_37::types::Evidence as RawEvidence;
                let mut state = serializer.serialize_struct("Request", 2)?;
                state.serialize_field("evidence", &RawEvidence::from(self.evidence.clone()))?;
                state.end()
            },
        }
    }
}

// This is a terrible hack for the fact that the serialization of the evidence
// changed between Tendermint 0.34 and 0.37.
impl<'de> Deserialize<'de> for Request {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        struct InnerV0_34 {
            evidence: tendermint_proto::v0_34::types::Evidence,
        }

        #[derive(Deserialize)]
        struct InnerV0_37 {
            evidence: tendermint_proto::v0_37::types::Evidence,
        }

        let value = serde_json::Value::deserialize(deserializer)?;

        serde_json::from_value::<InnerV0_34>(value.clone())
            .map_err(D::Error::custom)
            .and_then(|inner| Evidence::try_from(inner.evidence).map_err(D::Error::custom))
            .map(|evidence| Request {
                evidence,
                compat_mode: CompatMode::V0_34,
            })
            .or_else(|_| {
                serde_json::from_value::<InnerV0_37>(value)
                    .map_err(serde::de::Error::custom)
                    .and_then(|inner| Evidence::try_from(inner.evidence).map_err(D::Error::custom))
                    .map(|evidence| Request {
                        evidence,
                        compat_mode: CompatMode::V0_37,
                    })
            })
    }
}

impl Request {
    /// Create a new evidence broadcast RPC request
    pub fn new(evidence: Evidence, compat_mode: CompatMode) -> Request {
        Request {
            evidence,
            compat_mode,
        }
    }
}

impl RequestMessage for Request {
    fn method(&self) -> Method {
        Method::BroadcastEvidence
    }
}

impl<S: Dialect> crate::Request<S> for Request {
    type Response = Response;
}

impl<S: Dialect> crate::SimpleRequest<S> for Request {
    type Output = Response;
}

/// Response from either an evidence broadcast request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Evidence hash
    #[serde(with = "crate::serializers::tm_hash_base64")]
    pub hash: Hash,
}

impl crate::Response for Response {}
