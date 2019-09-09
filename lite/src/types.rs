/// TrustedState stores the latest state trusted by a lite client,
/// including the last header and the validator set to use to verify
/// the next header.
pub struct TrustedState<H, V>
where
    H: Header,
    V: Validators,
{
    pub header: H,
    pub next_validators: V,
}

/// Need to do something better here :)
pub type Height = u64;
pub type Hash = u64; // TODO
pub type Time = u64; // TODO
pub type Bytes = u64; // TODO

/// Header contains meta data about the block -
/// the height, the time, and the hash of the validator set
/// that should sign this header, and the hash of the validator
/// set that should sign the next header.
pub trait Header {
    fn height(&self) -> Height;
    fn bft_time(&self) -> Time;
    fn validators_hash(&self) -> Hash;
    fn next_validators_hash(&self) -> Hash;

    /// Hash of the header is the hash of the block!
    fn hash(&self) -> Hash;
}

/// Validators is the full validator set.
/// It has a hash and an underlying Validator type,
/// which should know its ID, voting power,
/// and be able to verify its signatures (ie. have access to the public key).
pub trait Validators {
    type Validator: Validator;

    /// Hash of the validator set.
    fn hash(&self) -> Hash;

    /// For iterating over the underlying validators.
    fn into_vec(&self) -> Vec<Self::Validator>;
}

/// Validator has an id, a voting power, and can verify
/// its own signatures. Note it must have implicit access
/// to its public key material (ie. the pre-image of the id)
/// to verify signatures.
pub trait Validator {
    fn power(&self) -> u64;
    fn verify(&self, sign_bytes: Bytes, signature: Bytes) -> bool;
}

/// Commit is proof a Header is valid.
/// It has an underlying Vote type with the relevant vote data
/// for verification.
pub trait Commit {
    type Vote: Vote;

    /// Hash of the header this commit is for.
    fn header_hash(&self) -> Hash;

    /// Return the underlying votes for iteration.
    /// The Vote is either for the right block id,
    /// or it's None.
    fn into_vec(&self) -> Vec<Option<Self::Vote>>;
}

/// Vote contains the data to verify a validator voted correctly in the commit.
/// In an ideal world, votes contain only signatures, and all votes are for the same
/// message. For now, Tendermint votes also sign over the validator's local timestamp,
/// so each vote is for a slightly different message.
pub trait Vote {
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
