//! ABCI Merkle proofs

use crate::serializers;
use serde::{Deserialize, Serialize};

/// Proof is Merkle proof defined by the list of ProofOps
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Proof {
    /// The list of ProofOps
    pub ops: Vec<ProofOp>,
}

/// ProofOp defines an operation used for calculating Merkle root
/// The data could be arbitrary format, providing necessary data
/// for example neighbouring node hash
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ProofOp {
    /// Type of the ProofOp
    #[serde(alias = "type")]
    pub field_type: String,
    /// Key of the ProofOp
    #[serde(
        default,
        serialize_with = "serializers::serialize_base64",
        deserialize_with = "serializers::parse_base64"
    )]
    pub key: Vec<u8>,
    /// Actual data
    #[serde(
        default,
        serialize_with = "serializers::serialize_base64",
        deserialize_with = "serializers::parse_base64"
    )]
    pub data: Vec<u8>,
}

#[cfg(test)]
mod test {
    use super::Proof;
    use crate::test::test_serialization_roundtrip;

    #[test]
    fn serialization_roundtrip() {
        let payload = r#"
        {
            "ops": [
                {
                    "type": "iavl:v",
                    "key": "Y29uc2Vuc3VzU3RhdGUvaWJjb25lY2xpZW50LzIy",
                    "data": "8QEK7gEKKAgIEAwYHCIgG9RAkJgHlxNjmyzOW6bUAidhiRSja0x6+GXCVENPG1oKKAgGEAUYFyIgwRns+dJvjf1Zk2BaFrXz8inPbvYHB7xx2HCy9ima5f8KKAgEEAMYFyogOr8EGajEV6fG5fzJ2fAAvVMgRLhdMJTzCPlogl9rxlIKKAgCEAIYFyIgcjzX/a+2bFbnNldpawQqZ+kYhIwz5r4wCUzuu1IFW04aRAoeY29uc2Vuc3VzU3RhdGUvaWJjb25lY2xpZW50LzIyEiAZ1uuG60K4NHJZZMuS9QX6o4eEhica5jIHYwflRiYkDBgX"
                },
                {
                    "type": "multistore",
                    "key": "aWJj",
                    "data": "CvEECjAKBGJhbmsSKAomCIjYAxIg2MEyyonbZButYnvSRkf2bPQg+nqA+Am1MeDxG6F4p1UKLwoDYWNjEigKJgiI2AMSIN2YHczeuXNvyetrSFQpkCcJzfB6PXVCw0i/XShMgPnIChEKB3VwZ3JhZGUSBgoECIjYAwovCgNnb3YSKAomCIjYAxIgYM0TfBli7KxhY4nWgDSDPykhUJwtKFql9RU5l86WinQKLwoDaWJjEigKJgiI2AMSIFp6aJASeInQKF8y824zjmgcFORN6M+ECbgFfJkobKs8CjAKBG1haW4SKAomCIjYAxIgsZzwmLQ7PH1UeZ/vCUSqlQmfgt3CGfoMgJLkUqKCv0EKMwoHc3Rha2luZxIoCiYIiNgDEiCiBZoBLyDGj5euy3n33ik+SpqYK9eB5xbI+iY8ycYVbwo0CghzbGFzaGluZxIoCiYIiNgDEiAJz3gEYuIhdensHU3b5qH5ons2quepd6EaRgCHXab6PQoyCgZzdXBwbHkSKAomCIjYAxIglWLA5/THPTiTxAlaLHOBYFIzEJTmKPznItUwAc8zD+AKEgoIZXZpZGVuY2USBgoECIjYAwowCgRtaW50EigKJgiI2AMSIMS8dZ1j8F6JVVv+hB1rHBZC+gIFJxHan2hM8qDC64n/CjIKBnBhcmFtcxIoCiYIiNgDEiB8VIzExUHX+SvHZFz/P9NM9THnw/gTDDLVReuZX8htLgo4CgxkaXN0cmlidXRpb24SKAomCIjYAxIg3u/Nd4L+8LT8OXJCh14o8PHIJ/GLQwsmE7KYIl1GdSYKEgoIdHJhbnNmZXISBgoECIjYAw=="
                }
            ]
        }"#;
        test_serialization_roundtrip::<Proof>(payload);
    }
}
