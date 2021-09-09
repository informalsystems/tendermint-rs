//! Tendermint consensus parameters

use crate::error::Error;
use crate::prelude::*;
use crate::{block, evidence, public_key};
use core::convert::{TryFrom, TryInto};
use serde::{Deserialize, Serialize};
use tendermint_proto::types::ConsensusParams as RawParams;
use tendermint_proto::types::ValidatorParams as RawValidatorParams;
use tendermint_proto::types::VersionParams as RawVersionParams;
use tendermint_proto::Protobuf;

/// Tendermint consensus parameters
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Params {
    /// Block size parameters
    pub block: block::Size,

    /// Evidence parameters
    pub evidence: evidence::Params,

    /// Validator parameters
    pub validator: ValidatorParams,

    /// Version parameters
    #[serde(skip)] // Todo: FIXME kvstore /genesis returns '{}' instead of '{app_version: "0"}'
    pub version: Option<VersionParams>,
}

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

/// Validator consensus parameters
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ValidatorParams {
    /// Allowed algorithms for validator signing
    pub pub_key_types: Vec<public_key::Algorithm>,
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

/// Version Parameters
#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Default)]
pub struct VersionParams {
    #[serde(with = "crate::serializers::from_str")]
    app_version: u64,
}

impl Protobuf<RawVersionParams> for VersionParams {}

impl TryFrom<RawVersionParams> for VersionParams {
    type Error = Error;

    fn try_from(value: RawVersionParams) -> Result<Self, Self::Error> {
        Ok(Self {
            app_version: value.app_version,
        })
    }
}

impl From<VersionParams> for RawVersionParams {
    fn from(value: VersionParams) -> Self {
        RawVersionParams {
            app_version: value.app_version,
        }
    }
}
