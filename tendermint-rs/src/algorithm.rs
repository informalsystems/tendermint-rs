//! Algorithm registry

/// Hash algorithms
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HashAlgorithm {
    /// SHA-256
    Sha256,
}

/// Digital signature algorithms
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SignatureAlgorithm {
    /// ECDSA over secp256k1
    EcdsaSecp256k1,

    /// EdDSA over Curve25519
    Ed25519,
}
