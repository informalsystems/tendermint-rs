//! Tendermint consensus parameters

use crate::{block, evidence, public_key};
use serde::{Deserialize, Serialize};

/// Tendermint consensus parameters
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Params {
    /// Block size parameters
    pub block: block::Size,

    /// Evidence parameters
    pub evidence: evidence::Params,

    /// Validator parameters
    pub validator: ValidatorParams,
}

/// Validator consensus parameters
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ValidatorParams {
    /// Allowed algorithms for validator signing
    pub pub_key_types: Vec<public_key::Algorithm>,
}
