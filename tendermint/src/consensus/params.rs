//! Tendermint consensus parameters

use serde::{Deserialize, Serialize};

use crate::{block, evidence, prelude::*, public_key};

/// All consensus-relevant parameters that can be adjusted by the ABCI app.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#consensusparams)
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Params {
    /// Parameters limiting the size of a block and time between consecutive blocks.
    pub block: block::Size,
    /// Parameters limiting the validity of evidence of byzantine behaviour.
    pub evidence: evidence::Params,
    /// Parameters limiting the types of public keys validators can use.
    pub validator: ValidatorParams,
    /// The ABCI application version.
    #[serde(skip)] // FIXME: kvstore /genesis returns '{}' instead of '{app_version: "0"}'
    pub version: Option<VersionParams>,
    /// Parameters specific to the Application Blockchain Interface.
    ///
    /// This field has been added in CometBFT 0.38 and will be ignored when
    /// encoding into earlier protocol versions.
    #[serde(default)]
    pub abci: AbciParams,
}

/// ValidatorParams restrict the public key types validators can use.
///
/// [Tendermint documentation](https://docs.tendermint.com/master/spec/core/data_structures.html#validatorparams)
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ValidatorParams {
    /// List of accepted public key types.
    pub pub_key_types: Vec<public_key::Algorithm>,
}

/// Version Parameters
///
/// [Tendermint documentation](https://docs.tendermint.com/master/spec/core/data_structures.html#versionparams)
#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Default)]
pub struct VersionParams {
    /// The ABCI application version.
    #[serde(with = "crate::serializers::from_str")]
    pub app: u64,
}

/// Parameters specific to the Application Blockchain Interface.
#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Default)]
pub struct AbciParams {
    /// Configures the first height during which
    /// vote extensions will be enabled. During this specified height, and for all
    /// subsequent heights, precommit messages that do not contain valid extension data
    /// will be considered invalid. Prior to this height, vote extensions will not
    /// be used or accepted by validators on the network.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vote_extensions_enable_height: Option<block::Height>,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// Todo: How are these key types created?
fn key_type(s: &str) -> public_key::Algorithm {
    if s == "Ed25519" || s == "ed25519" {
        return public_key::Algorithm::Ed25519;
    }
    if s == "Secp256k1" || s == "secp256k1" {
        return public_key::Algorithm::Secp256k1;
    }
    public_key::Algorithm::Ed25519 // Todo: Shall we error out for invalid key types?
}

mod v0_34 {
    use tendermint_proto::v0_34::{
        abci::ConsensusParams as RawAbciConsensusParams,
        types::{
            ConsensusParams as RawParams, ValidatorParams as RawValidatorParams,
            VersionParams as RawVersionParams,
        },
    };
    use tendermint_proto::Protobuf;

    use super::{key_type, Params, ValidatorParams, VersionParams};
    use crate::{error::Error, prelude::*, public_key};

    impl Protobuf<RawParams> for Params {}

    impl TryFrom<RawParams> for Params {
        type Error = Error;

        fn try_from(value: RawParams) -> Result<Self, Self::Error> {
            Ok(Self {
                block: value
                    .block
                    .ok_or_else(|| Error::invalid_block("missing block".to_string()))?
                    .try_into()?,
                evidence: value
                    .evidence
                    .ok_or_else(Error::invalid_evidence)?
                    .try_into()?,
                validator: value
                    .validator
                    .ok_or_else(Error::invalid_validator_params)?
                    .try_into()?,
                version: value.version.map(TryFrom::try_from).transpose()?,
                abci: Default::default(),
            })
        }
    }

    impl From<Params> for RawParams {
        fn from(value: Params) -> Self {
            RawParams {
                block: Some(value.block.into()),
                evidence: Some(value.evidence.into()),
                validator: Some(value.validator.into()),
                version: value.version.map(From::from),
            }
        }
    }

    impl Protobuf<RawAbciConsensusParams> for Params {}

    impl TryFrom<RawAbciConsensusParams> for Params {
        type Error = Error;

        fn try_from(value: RawAbciConsensusParams) -> Result<Self, Self::Error> {
            Ok(Self {
                block: value
                    .block
                    .ok_or_else(|| Error::invalid_block("missing block".to_string()))?
                    .try_into()?,
                evidence: value
                    .evidence
                    .ok_or_else(Error::invalid_evidence)?
                    .try_into()?,
                validator: value
                    .validator
                    .ok_or_else(Error::invalid_validator_params)?
                    .try_into()?,
                version: value.version.map(TryFrom::try_from).transpose()?,
                abci: Default::default(),
            })
        }
    }

    impl From<Params> for RawAbciConsensusParams {
        fn from(value: Params) -> Self {
            RawAbciConsensusParams {
                block: Some(value.block.into()),
                evidence: Some(value.evidence.into()),
                validator: Some(value.validator.into()),
                version: value.version.map(From::from),
            }
        }
    }

    impl Protobuf<RawValidatorParams> for ValidatorParams {}

    impl TryFrom<RawValidatorParams> for ValidatorParams {
        type Error = Error;

        fn try_from(value: RawValidatorParams) -> Result<Self, Self::Error> {
            Ok(Self {
                pub_key_types: value.pub_key_types.iter().map(|f| key_type(f)).collect(),
            })
        }
    }

    impl From<ValidatorParams> for RawValidatorParams {
        fn from(value: ValidatorParams) -> Self {
            RawValidatorParams {
                pub_key_types: value
                    .pub_key_types
                    .into_iter()
                    .map(|k| match k {
                        public_key::Algorithm::Ed25519 => "ed25519".to_string(),
                        public_key::Algorithm::Secp256k1 => "secp256k1".to_string(),
                    })
                    .collect(),
            }
        }
    }

    impl Protobuf<RawVersionParams> for VersionParams {}

    impl TryFrom<RawVersionParams> for VersionParams {
        type Error = Error;

        fn try_from(value: RawVersionParams) -> Result<Self, Self::Error> {
            Ok(Self {
                app: value.app_version,
            })
        }
    }

    impl From<VersionParams> for RawVersionParams {
        fn from(value: VersionParams) -> Self {
            RawVersionParams {
                app_version: value.app,
            }
        }
    }
}

mod v0_37 {
    use tendermint_proto::v0_37::types::{
        ConsensusParams as RawParams, ValidatorParams as RawValidatorParams,
        VersionParams as RawVersionParams,
    };
    use tendermint_proto::Protobuf;

    use super::{key_type, Params, ValidatorParams, VersionParams};
    use crate::{error::Error, prelude::*, public_key};

    impl Protobuf<RawParams> for Params {}

    impl TryFrom<RawParams> for Params {
        type Error = Error;

        fn try_from(value: RawParams) -> Result<Self, Self::Error> {
            Ok(Self {
                block: value
                    .block
                    .ok_or_else(|| Error::invalid_block("missing block".to_string()))?
                    .try_into()?,
                evidence: value
                    .evidence
                    .ok_or_else(Error::invalid_evidence)?
                    .try_into()?,
                validator: value
                    .validator
                    .ok_or_else(Error::invalid_validator_params)?
                    .try_into()?,
                version: value.version.map(TryFrom::try_from).transpose()?,
                abci: Default::default(),
            })
        }
    }

    impl From<Params> for RawParams {
        fn from(value: Params) -> Self {
            RawParams {
                block: Some(value.block.into()),
                evidence: Some(value.evidence.into()),
                validator: Some(value.validator.into()),
                version: value.version.map(From::from),
            }
        }
    }

    impl Protobuf<RawValidatorParams> for ValidatorParams {}

    impl TryFrom<RawValidatorParams> for ValidatorParams {
        type Error = Error;

        fn try_from(value: RawValidatorParams) -> Result<Self, Self::Error> {
            Ok(Self {
                pub_key_types: value.pub_key_types.iter().map(|f| key_type(f)).collect(),
            })
        }
    }

    impl From<ValidatorParams> for RawValidatorParams {
        fn from(value: ValidatorParams) -> Self {
            RawValidatorParams {
                pub_key_types: value
                    .pub_key_types
                    .into_iter()
                    .map(|k| match k {
                        public_key::Algorithm::Ed25519 => "ed25519".to_string(),
                        public_key::Algorithm::Secp256k1 => "secp256k1".to_string(),
                    })
                    .collect(),
            }
        }
    }

    impl Protobuf<RawVersionParams> for VersionParams {}

    impl TryFrom<RawVersionParams> for VersionParams {
        type Error = Error;

        fn try_from(value: RawVersionParams) -> Result<Self, Self::Error> {
            Ok(Self { app: value.app })
        }
    }

    impl From<VersionParams> for RawVersionParams {
        fn from(value: VersionParams) -> Self {
            RawVersionParams { app: value.app }
        }
    }
}

mod v0_38 {
    use tendermint_proto::v0_38::types::{
        AbciParams as RawAbciParams, ConsensusParams as RawParams,
        ValidatorParams as RawValidatorParams, VersionParams as RawVersionParams,
    };
    use tendermint_proto::Protobuf;

    use super::{key_type, AbciParams, Params, ValidatorParams, VersionParams};
    use crate::{error::Error, prelude::*, public_key};

    impl Protobuf<RawParams> for Params {}

    impl TryFrom<RawParams> for Params {
        type Error = Error;

        fn try_from(value: RawParams) -> Result<Self, Self::Error> {
            Ok(Self {
                block: value
                    .block
                    .ok_or_else(|| Error::invalid_block("missing block".to_string()))?
                    .try_into()?,
                evidence: value
                    .evidence
                    .ok_or_else(Error::invalid_evidence)?
                    .try_into()?,
                validator: value
                    .validator
                    .ok_or_else(Error::invalid_validator_params)?
                    .try_into()?,
                version: value.version.map(TryFrom::try_from).transpose()?,
                abci: value
                    .abci
                    .map(TryFrom::try_from)
                    .transpose()?
                    .unwrap_or_default(),
            })
        }
    }

    impl From<Params> for RawParams {
        fn from(value: Params) -> Self {
            RawParams {
                block: Some(value.block.into()),
                evidence: Some(value.evidence.into()),
                validator: Some(value.validator.into()),
                version: value.version.map(From::from),
                abci: Some(value.abci.into()),
            }
        }
    }

    impl Protobuf<RawValidatorParams> for ValidatorParams {}

    impl TryFrom<RawValidatorParams> for ValidatorParams {
        type Error = Error;

        fn try_from(value: RawValidatorParams) -> Result<Self, Self::Error> {
            Ok(Self {
                pub_key_types: value.pub_key_types.iter().map(|f| key_type(f)).collect(),
            })
        }
    }

    impl From<ValidatorParams> for RawValidatorParams {
        fn from(value: ValidatorParams) -> Self {
            RawValidatorParams {
                pub_key_types: value
                    .pub_key_types
                    .into_iter()
                    .map(|k| match k {
                        public_key::Algorithm::Ed25519 => "ed25519".to_string(),
                        public_key::Algorithm::Secp256k1 => "secp256k1".to_string(),
                    })
                    .collect(),
            }
        }
    }

    impl Protobuf<RawVersionParams> for VersionParams {}

    impl TryFrom<RawVersionParams> for VersionParams {
        type Error = Error;

        fn try_from(value: RawVersionParams) -> Result<Self, Self::Error> {
            Ok(Self { app: value.app })
        }
    }

    impl From<VersionParams> for RawVersionParams {
        fn from(value: VersionParams) -> Self {
            RawVersionParams { app: value.app }
        }
    }

    impl Protobuf<RawAbciParams> for AbciParams {}

    impl TryFrom<RawAbciParams> for AbciParams {
        type Error = Error;

        fn try_from(message: RawAbciParams) -> Result<Self, Self::Error> {
            let vote_extensions_enable_height = match message.vote_extensions_enable_height {
                0 => None,
                h => Some(h.try_into()?),
            };
            Ok(Self {
                vote_extensions_enable_height,
            })
        }
    }

    impl From<AbciParams> for RawAbciParams {
        fn from(value: AbciParams) -> Self {
            Self {
                vote_extensions_enable_height: value
                    .vote_extensions_enable_height
                    .map_or(0, Into::into),
            }
        }
    }
}
