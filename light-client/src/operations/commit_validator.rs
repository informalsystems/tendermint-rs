//! Provides an interface and default implementation for the `CommitValidator` operation

use crate::{
    operations::{Hasher, ProdHasher},
    predicates::errors::{self as error, VerificationError},
    types::{SignedHeader, ValidatorSet},
};

use tendermint::block::CommitSig;

/// Validates the commit associated with a header against a validator set
pub trait CommitValidator: Send + Sync {
    /// Perform basic validation
    fn validate(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<(), VerificationError>;

    /// Perform full validation, only necessary if we do full verification (2/3)
    fn validate_full(
        &self,
        signed_header: &SignedHeader,
        validator_set: &ValidatorSet,
    ) -> Result<(), VerificationError>;
}

/// Production-ready implementation of a commit validator
pub struct ProdCommitValidator {
    hasher: Box<dyn Hasher>,
}

impl ProdCommitValidator {
    /// Create a new commit validator using the given [`Hasher`]
    /// to compute the hash of headers and validator sets.
    pub fn new(hasher: impl Hasher + 'static) -> Self {
        Self {
            hasher: Box::new(hasher),
        }
    }
}

impl Default for ProdCommitValidator {
    fn default() -> Self {
        Self::new(ProdHasher::default())
    }
}

impl CommitValidator for ProdCommitValidator {
    fn validate(
        &self,
        signed_header: &SignedHeader,
        validator_set: &ValidatorSet,
    ) -> Result<(), VerificationError> {
        let signatures = &signed_header.commit.signatures;

        // Check the the commit contains at least one non-absent signature.
        // See https://github.com/informalsystems/tendermint-rs/issues/650
        let has_present_signatures = signatures.iter().any(|cs| !cs.is_absent());
        if !has_present_signatures {
            return Err(error::no_signature_for_commit_error());
        }

        // Check that that the number of signatures matches the number of validators.
        if signatures.len() != validator_set.validators().len() {
            return Err(error::mismatch_pre_commit_length_error(
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
    fn validate_full(
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

            if validator_set.validator(*validator_address) == None {
                return Err(error::faulty_signer_error(
                    *validator_address,
                    self.hasher.hash_validator_set(validator_set),
                ));
            }
        }

        Ok(())
    }
}
