//! Provides an interface and default implementation for the `CommitValidator` operation

use core::marker::PhantomData;

use tendermint::block::CommitSig;
#[cfg(feature = "rust-crypto")]
use tendermint::crypto::DefaultHostFunctionsManager;

use crate::{
    errors::VerificationError,
    operations::{Hasher, ProdHasher},
    types::{SignedHeader, ValidatorSet},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitValidator<C> {
    hasher: ProdHasher,
    _c: PhantomData<C>,
}

impl<C> CommitValidator<C> {
    pub fn validate(
        &self,
        signed_header: &SignedHeader,
        validator_set: &ValidatorSet,
    ) -> Result<(), VerificationError> {
        let signatures = &signed_header.commit.signatures;

        // Check the the commit contains at least one non-absent signature.
        // See https://github.com/informalsystems/tendermint-rs/issues/650
        let has_present_signatures = signatures.iter().any(|cs| !cs.is_absent());
        if !has_present_signatures {
            return Err(VerificationError::no_signature_for_commit());
        }

        // Check that that the number of signatures matches the number of validators.
        if signatures.len() != validator_set.validators().len() {
            return Err(VerificationError::mismatch_pre_commit_length(
                signatures.len(),
                validator_set.validators().len(),
            ));
        }

        Ok(())
    }

    // This check is only necessary if we do full verification (2/3)
    //
    // See https://github.com/informalsystems/tendermint-rs/issues/281
    //
    // It returns `ImplementationSpecific` error if it detects a signer
    // that is not present in the validator set
    pub fn validate_full(
        &self,
        signed_header: &SignedHeader,
        validator_set: &ValidatorSet,
    ) -> Result<(), VerificationError> {
        for commit_sig in signed_header.commit.signatures.iter() {
            let validator_address = match commit_sig {
                CommitSig::BlockIdFlagAbsent => continue,
                CommitSig::BlockIdFlagCommit {
                    validator_address, ..
                } => validator_address,
                CommitSig::BlockIdFlagNil {
                    validator_address, ..
                } => validator_address,
            };

            if validator_set.validator(*validator_address).is_none() {
                return Err(VerificationError::faulty_signer(
                    *validator_address,
                    self.hasher.hash_validator_set(validator_set),
                ));
            }
        }

        Ok(())
    }
}

/// The batteries-included validator, for when you don't mind the dependencies on
/// the full rust-crypto stack.
#[cfg(feature = "rust-crypto")]
pub type ProdCommitValidator = CommitValidator<DefaultHostFunctionsManager>;

#[cfg(not(feature = "rust-crypto"))]
/// Production-ready implementation of a commit validator
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProdCommitValidator<C> {
    inner: CommitValidator<C>,
}

#[cfg(not(feature = "rust-crypto"))]
impl<C> AsRef<CommitValidator<C>> for ProdCommitValidator<C> {
    fn as_ref(&self) -> &CommitValidator<C> {
        &self.inner
    }
}

#[cfg(feature = "rust-crypto")]
impl AsRef<CommitValidator<DefaultHostFunctionsManager>> for ProdCommitValidator {
    fn as_ref(&self) -> &CommitValidator<DefaultHostFunctionsManager> {
        self
    }
}

#[cfg(not(feature = "rust-crypto"))]
impl<C> ProdCommitValidator<C> {
    /// Create a new commit validator using the given [`Hasher`]
    /// to compute the hash of headers and validator sets.
    pub fn new(hasher: ProdHasher) -> Self {
        Self {
            inner: CommitValidator {
                hasher,
                _c: PhantomData::default(),
            },
        }
    }
}

#[cfg(feature = "rust-crypto")]
impl ProdCommitValidator {
    /// Create a new commit validator using the given [`Hasher`]
    /// to compute the hash of headers and validator sets.
    pub fn new(hasher: ProdHasher) -> Self {
        CommitValidator {
            hasher,
            _c: PhantomData::default(),
        }
    }
}

#[cfg(not(feature = "rust-crypto"))]
impl<C> Default for ProdCommitValidator<C> {
    fn default() -> Self {
        Self::new(ProdHasher::default())
    }
}

#[cfg(feature = "rust-crypto")]
impl Default for ProdCommitValidator {
    fn default() -> Self {
        Self::new(ProdHasher::default())
    }
}
