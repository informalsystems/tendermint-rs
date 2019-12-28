//! All traits that are necessary and need to be implemented to use the main
//! verification logic in `super::verifier` for a light client.

use crate::block::Height;
use crate::Hash;

use failure::_core::fmt::Debug;
use std::time::SystemTime;

/// TrustedState stores the latest state trusted by a lite client,
/// including the last header (at height h-1) and the validator set
/// (at height h) to use to verify the next header.
pub trait TrustedState {
    type LastHeader: SignedHeader;
    type ValidatorSet: ValidatorSet;

    /// Initialize the TrustedState with the given signed header and validator set.
    /// Note that if the height of the passed in header is h-1, the passed in validator set
    /// must have been requested for height h.
    fn new(last_header: &Self::LastHeader, vals: &Self::ValidatorSet) -> Self;

    fn last_header(&self) -> &Self::LastHeader; // height H-1
    fn validators(&self) -> &Self::ValidatorSet; // height H
}

/// SignedHeader bundles a Header and a Commit for convenience.
pub trait SignedHeader {
    type Header: Header;
    type Commit: Commit;

    fn header(&self) -> &Self::Header;
    fn commit(&self) -> &Self::Commit;
}

/// Header contains meta data about the block -
/// the height, the time, the hash of the validator set
/// that should sign this header, and the hash of the validator
/// set that should sign the next header.
pub trait Header: Debug {
    /// The header's notion of (bft-)time.
    /// We assume it can be converted to SystemTime.
    type Time: Into<SystemTime>;

    fn height(&self) -> Height;
    fn bft_time(&self) -> Self::Time;
    fn validators_hash(&self) -> Hash;
    fn next_validators_hash(&self) -> Hash;

    /// Hash of the header (ie. the hash of the block).
    fn hash(&self) -> Hash;
}

/// ValidatorSet is the full validator set.
/// It exposes its hash, which should match whats in a header,
/// and its total power. It also has an underlying
/// Validator type which can be used for verifying signatures.
/// It also provides a lookup method to fetch a validator by
/// its identifier.
pub trait ValidatorSet {
    /// Hash of the validator set.
    fn hash(&self) -> Hash;

    /// Total voting power of the set
    fn total_power(&self) -> u64;

    /// Return the number of validators in this validator set.
    fn len(&self) -> usize;

    /// Returns true iff the validator set is empty.
    fn is_empty(&self) -> bool;
}

/// Commit is proof a Header is valid.
/// It has an underlying Vote type with the relevant vote data
/// for verification.
pub trait Commit {
    type ValidatorSet: ValidatorSet;

    /// Hash of the header this commit is for.
    fn header_hash(&self) -> Hash;

    /// Compute the voting power of the validators that correctly signed the commit,
    /// have according to their voting power in the passed in validator set.
    /// Will return an error in case an invalid signature was included.
    ///
    /// This method corresponds to the (pure) auxiliary function int the spec:
    /// `votingpower_in(signers(h.Commit),h.Header.V)`.
    fn voting_power_in(&self, vals: &Self::ValidatorSet) -> Result<u64, Error>;

    /// Return the number of votes included in this commit
    /// (including nil/empty votes).
    fn votes_len(&self) -> usize;
}

/// TrustThreshold defines what fraction of the total voting power of a known
/// and trusted validator set is sufficient for a commit to be
/// accepted going forward.
/// The default implementation returns true, iff at least a third of the trusted
/// voting power signed (in other words at least one honest validator signed).
/// Some clients might require more than +1/3 and can implement their own
/// TrustLevel which can be passed into all relevant methods.
pub trait TrustThreshold {
    fn is_enough_power(&self, signed_voting_power: u64, total_voting_power: u64) -> bool {
        signed_voting_power * 3 > total_voting_power
    }
}

/// Requester can be used to request `SignedHeaders` and `ValidatorSet`s for a
/// given height, e.g., by talking to a tendermint fullnode through RPC.
pub trait Requester {
    // TODO(Liamsi): consider putting this trait and the Store into a separate module / file...
    type SignedHeader: SignedHeader;
    type ValidatorSet: ValidatorSet;

    /// Request the signed header at height h.
    fn signed_header<H>(&self, h: H) -> Result<Self::SignedHeader, Error>
    where
        H: Into<Height>;

    /// Request the validator set at height h.
    fn validator_set<H>(&self, h: H) -> Result<Self::ValidatorSet, Error>
    where
        H: Into<Height>;
}

/// This store can be used to store all the headers that have passed basic verification
/// and that are within the light client's trust period.
pub trait Store {
    type TrustedState: TrustedState;

    /// Add this state (header at height h, validators at height h+1) as trusted to the store.
    fn add(&mut self, trusted: &Self::TrustedState) -> Result<(), Error>;

    /// Retrieve the trusted state at height h if it exists.
    /// If it does not exist return an error.
    fn get(&self, h: Height) -> Result<&Self::TrustedState, Error>;

    /// Retrieve the trusted signed header with the largest height h' with h' <= h, if it exists.
    /// If it does not exist return an error.
    fn get_smaller_or_equal(&self, h: Height) -> Result<Self::TrustedState, Error>;
}

#[derive(Debug, PartialEq)]
pub enum Error {
    Expired,
    DurationOutOfRange,

    InvalidSignature, // TODO: deduplicate with ErrorKind::SignatureInvalid

    InvalidValidatorSet,
    InvalidNextValidatorSet,
    InvalidCommitValue, // commit is not for the header we expected
    InvalidCommitLength,

    InsufficientVotingPower, // TODO(Liamsi): change to same name as spec if this changes (curently ErrTooMuchChange)

    RequestFailed,
}
