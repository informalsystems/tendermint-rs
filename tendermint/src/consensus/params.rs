//! Tendermint consensus parameters

use crate::{block, evidence, public_key};
use crate::{Error, Kind};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use tendermint_proto::abci::ConsensusParams as RawParams;
use tendermint_proto::types::ValidatorParams as RawValidatorParams;
use tendermint_proto::types::VersionParams as RawVersionParams;
use tendermint_proto::DomainType;

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
    pub version: VersionParams,
}

impl DomainType<RawParams> for Params {}

impl TryFrom<RawParams> for Params {
    type Error = Error;

    fn try_from(value: RawParams) -> Result<Self, Self::Error> {
        Ok(Self {
            block: value.block.ok_or(Kind::InvalidBlock)?.try_into()?,
            evidence: value.evidence.ok_or(Kind::InvalidEvidence)?.try_into()?,
            validator: value
                .validator
                .ok_or(Kind::InvalidValidatorParams)?
                .try_into()?,
            version: value
                .version
                .ok_or(Kind::InvalidVersionParams)?
                .try_into()?,
        })
    }
}

impl From<Params> for RawParams {
    fn from(value: Params) -> Self {
        RawParams {
            block: Some(value.block.into()),
            evidence: Some(value.evidence.into()),
            validator: Some(value.validator.into()),
            version: Some(value.version.into()),
        }
    }
}

/// Validator consensus parameters
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ValidatorParams {
    /// Allowed algorithms for validator signing
    pub pub_key_types: Vec<public_key::Algorithm>,
}

impl DomainType<RawValidatorParams> for ValidatorParams {}

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
#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(transparent)]
pub struct VersionParams(#[serde(with = "crate::serializers::from_str")] u64);

impl DomainType<RawVersionParams> for VersionParams {}

impl TryFrom<RawVersionParams> for VersionParams {
    type Error = Error;

    fn try_from(value: RawVersionParams) -> Result<Self, Self::Error> {
        Ok(Self(value.app_version))
    }
}

impl From<VersionParams> for RawVersionParams {
    fn from(value: VersionParams) -> Self {
        RawVersionParams {
            app_version: value.value(),
        }
    }
}

impl VersionParams {
    /// constructor
    pub fn new(value: u64) -> Self {
        VersionParams(value)
    }
    /// Getter
    pub fn value(&self) -> u64 {
        self.0
    }
}
