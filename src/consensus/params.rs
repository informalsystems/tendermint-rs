//! Tendermint consensus parameters

use crate::{block, evidence, public_key};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Tendermint consensus parameters
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Params {
    /// Block size parameters
    pub block: block::Size,

    /// Evidence parameters
    pub evidence: evidence::Params,

    /// Validator parameters
    pub validator: ValidatorParams,
}

/// Validator consensus parameters
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorParams {
    /// Allowed algorithms for validator signing
    pub pub_key_types: Vec<public_key::Algorithm>,
}
