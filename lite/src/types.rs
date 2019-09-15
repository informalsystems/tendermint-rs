/// TrustedState stores the latest state trusted by a lite client,
/// including the last header and the validator set to use to verify
/// the next header.
pub struct TrustedState<H, V>
where
    H: Header,
    V: Validators,
{
    pub last_header: H, // height H-1
    pub validators: V,  // height H
}

/// Need to do something better here :)
pub type Height = u64;
pub type Hash = u64; // TODO
pub type Time = u64; // TODO
pub type Bytes = u64; // TODO
pub type ValID = u64; // TODO

/// Header contains meta data about the block -
/// the height, the time, the hash of the validator set
/// that should sign this header, and the hash of the validator
/// set that should sign the next header.
pub trait Header {
    fn height(&self) -> Height;
    fn bft_time(&self) -> Time;
    fn validators_hash(&self) -> Hash;
    fn next_validators_hash(&self) -> Hash;

    /// Hash of the header (ie. the hash of the block).
    fn hash(&self) -> Hash;
}

/// Validators is the full validator set.
/// It exposes its hash, which should match whats in a header,
/// and its total power. It also has an underlying
/// Validator type which can be used for verifying signatures.
pub trait Validators {
    type Validator: Validator;

    /// Hash of the validator set.
    fn hash(&self) -> Hash;

    /// Total voting power of the set
    fn total_power(&self) -> u64;

    /// For iterating over the underlying validators.
    fn into_vec(&self) -> Vec<Self::Validator>;
}

/// ValidatorsLookup allows validator to be fetched via their ID
/// (ie. their address).
pub trait ValidatorsLookup: Validators {
    fn validator(&self, val_id: ValID) -> Option<Self::Validator>;
}

/// Validator has a voting power and can verify
/// its own signatures. Note it must have implicit access
/// to its public key material to verify signatures.
pub trait Validator {
    fn power(&self) -> u64;
    fn verify_signature(&self, sign_bytes: Bytes, signature: Bytes) -> bool;
}

/// Commit is proof a Header is valid.
/// It has an underlying Vote type with the relevant vote data
/// for verification.
pub trait Commit {
    type Vote: Vote;

    /// Hash of the header this commit is for.
    fn header_hash(&self) -> Hash;

    /// Return the underlying votes for iteration.
    /// All votes here are for the correct block id -
    /// we ignore absent votes and votes for nil here.
    /// NOTE: we may want to check signatures for nil votes,
    /// and thus use an ternary enum here instead of the binary Option.
    fn into_vec(&self) -> Vec<Option<Self::Vote>>;
}

/// Vote contains the data to verify a validator voted correctly in the commit.
/// In an ideal world, votes contain only signatures, and all votes are for the same
/// message. For now, Tendermint votes also sign over the validator's local timestamp,
/// so each vote is for a slightly different message.
/// Note the Vote must also know which validator it is from.
pub trait Vote {
    fn validator_id(&self) -> ValID;
    fn sign_bytes(&self) -> Bytes;
    fn signature(&self) -> Bytes;
}

pub enum Error {
    Expired,
    NonSequentialHeight,

    InvalidValidators,
    InvalidNextValidators,
    InvalidCommitValue, // commit is not for the header we expected
    InvalidSignature,

    InsufficientVotingPower,
}
