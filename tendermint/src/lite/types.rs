//! All traits that are necessary and need to be implemented to use the main
//! verification logic in [`super::verifier`] for a light client.

use crate::Hash;

use crate::lite::error::{Error, ErrorKind};
use failure::_core::fmt::Debug;
use std::time::SystemTime;

pub type Height = u64;

/// Header contains meta data about the block -
/// the height, the time, the hash of the validator set
/// that should sign this header, and the hash of the validator
/// set that should sign the next header.
pub trait Header: Clone {
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
/// It exposes its hash and its total power.
pub trait ValidatorSet: Clone {
    /// Hash of the validator set.
    fn hash(&self) -> Hash;

    /// Total voting power of the set
    fn total_power(&self) -> u64;
}

/// Commit is used to prove a Header can be trusted.
/// Verifying the Commit requires access to an associated ValidatorSet
/// to determine what voting power signed the commit.
pub trait Commit: Clone {
    type ValidatorSet: ValidatorSet;

    /// Hash of the header this commit is for.
    fn header_hash(&self) -> Hash;

    /// Compute the voting power of the validators that correctly signed the commit,
    /// according to their voting power in the passed in validator set.
    /// Will return an error in case an invalid signature was included.
    /// TODO/XXX: This cannot detect if a signature from an incorrect validator
    /// is included. That's fine when we're just trying to see if we can skip,
    /// but when actually verifying it means we might accept commits that have sigs from
    /// outside the correct validator set, which is something we expect to be able to detect
    /// (it's not a real issue, but it would indicate a faulty full node).
    ///
    ///
    /// This method corresponds to the (pure) auxiliary function in the spec:
    /// `votingpower_in(signers(h.Commit),h.Header.V)`.
    /// Note this expects the Commit to be able to compute `signers(h.Commit)`,
    /// ie. the identity of the validators that signed it, so they
    /// can be cross-referenced with the given `vals`.
    fn voting_power_in(&self, vals: &Self::ValidatorSet) -> Result<u64, Error>;

    /// Implementers should add addition validation against the given validator set
    /// or other implementation specific validation here.
    /// E.g. validate that the length of the included signatures in the commit match
    /// with the number of validators.
    fn validate(&self, vals: &Self::ValidatorSet) -> Result<(), Error>;
}

/// TrustThreshold defines how much of the total voting power of a known
/// and trusted validator set is sufficient for a commit to be
/// accepted going forward.
pub trait TrustThreshold: Copy + Clone + Debug {
    fn is_enough_power(&self, signed_voting_power: u64, total_voting_power: u64) -> bool;
}

/// TrustThresholdFraction defines what fraction of the total voting power of a known
/// and trusted validator set is sufficient for a commit to be
/// accepted going forward.
/// The [`Default::default()`] returns true, iff at least a third of the trusted
/// voting power signed (in other words at least one honest validator signed).
/// Some clients might require more than +1/3 and can implement their own
/// [`TrustThreshold`] which can be passed into all relevant methods.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TrustThresholdFraction {
    numerator: u64,
    denominator: u64,
}

impl TrustThresholdFraction {
    pub fn new(numerator: u64, denominator: u64) -> Result<Self, ErrorKind> {
        if numerator <= denominator && denominator > 0 {
            return Ok(Self {
                numerator,
                denominator,
            });
        }
        Err(ErrorKind::InvalidTrustThreshold {
            got: format!("{}/{}", numerator, denominator),
        })
    }
}

// TODO: should this go in the central place all impls live instead? (currently lite_impl)
impl TrustThreshold for TrustThresholdFraction {
    fn is_enough_power(&self, signed_voting_power: u64, total_voting_power: u64) -> bool {
        signed_voting_power * self.denominator > total_voting_power * self.numerator
    }
}

impl Default for TrustThresholdFraction {
    fn default() -> Self {
        Self::new(1, 3)
            .expect("initializing TrustThresholdFraction with valid fraction mustn't panic")
    }
}

/// Requester can be used to request [`SignedHeader`]s and [`ValidatorSet`]s for a
/// given height, e.g., by talking to a tendermint fullnode through RPC.
pub trait Requester<C, H>
where
    C: Commit,
    H: Header,
{
    /// Request the [`SignedHeader`] at height h.
    fn signed_header(&self, h: Height) -> Result<SignedHeader<C, H>, ErrorKind>;

    /// Request the validator set at height h.
    fn validator_set(&self, h: Height) -> Result<C::ValidatorSet, ErrorKind>;
}

/// TrustedState contains a state trusted by a lite client,
/// including the last header (at height h-1) and the validator set
/// (at height h) to use to verify the next header.
#[derive(Clone, Debug, PartialEq)]
pub struct TrustedState<C, H>
where
    H: Header,
    C: Commit,
{
    last_header: SignedHeader<C, H>, // height H-1
    validators: C::ValidatorSet,     // height H
}

impl<C, H> TrustedState<C, H>
where
    H: Header,
    C: Commit,
{
    /// Initialize the TrustedState with the given signed header and validator set.
    /// Note that if the height of the passed in header is h-1, the passed in validator set
    /// must have been requested for height h.
    pub fn new(last_header: &SignedHeader<C, H>, validators: &C::ValidatorSet) -> Self {
        Self {
            last_header: last_header.clone(),
            validators: validators.clone(),
        }
    }

    pub fn last_header(&self) -> &SignedHeader<C, H> {
        &self.last_header
    }

    pub fn validators(&self) -> &C::ValidatorSet {
        &self.validators
    }
}

/// SignedHeader bundles a [`Header`] and a [`Commit`] for convenience.
#[derive(Clone, Debug, PartialEq)] // NOTE: Copy/Clone/Debug for convenience in testing ...
pub struct SignedHeader<C, H>
where
    C: Commit,
    H: Header,
{
    commit: C,
    header: H,
}

impl<C, H> SignedHeader<C, H>
where
    C: Commit,
    H: Header,
{
    pub fn new(commit: C, header: H) -> Self {
        Self { commit, header }
    }

    pub fn commit(&self) -> &C {
        &self.commit
    }

    pub fn header(&self) -> &H {
        &self.header
    }
}

pub(super) mod mocks {
    use serde::Serialize;
    use sha2::{Digest, Sha256};

    use crate::{hash::Algorithm, Hash};

    use super::*;
    use std::collections::HashMap;

    #[derive(Clone, Debug, PartialEq, Serialize)]
    pub struct MockHeader {
        height: u64,
        time: SystemTime,
        vals: Hash,
        next_vals: Hash,
    }

    impl MockHeader {
        pub fn new(height: u64, time: SystemTime, vals: Hash, next_vals: Hash) -> MockHeader {
            MockHeader {
                height,
                time,
                vals,
                next_vals,
            }
        }
    }

    impl Header for MockHeader {
        type Time = SystemTime;

        fn height(&self) -> Height {
            self.height
        }
        fn bft_time(&self) -> Self::Time {
            self.time
        }
        fn validators_hash(&self) -> Hash {
            self.vals
        }
        fn next_validators_hash(&self) -> Hash {
            self.next_vals
        }
        fn hash(&self) -> Hash {
            json_hash(self)
        }
    }

    pub fn json_hash<T: ?Sized + Serialize>(value: &T) -> Hash {
        let encoded = serde_json::to_vec(value).unwrap();
        let hashed = Sha256::digest(&encoded);
        Hash::new(Algorithm::Sha256, &hashed).unwrap()
    }

    // vals are just ints, each has power 1
    #[derive(Clone, Debug, PartialEq, Serialize)]
    pub struct MockValSet {
        // NOTE: use HashSet instead?
        vals: Vec<usize>,
    }

    impl MockValSet {
        pub fn new(vals: Vec<usize>) -> MockValSet {
            MockValSet { vals }
        }
    }

    impl ValidatorSet for MockValSet {
        fn hash(&self) -> Hash {
            json_hash(&self)
        }
        fn total_power(&self) -> u64 {
            self.vals.len() as u64
        }
    }

    // commit is a list of vals that signed.
    #[derive(Clone, Debug, PartialEq, Serialize)]
    pub struct MockCommit {
        hash: Hash,
        vals: Vec<usize>,
    }

    impl MockCommit {
        pub fn new(hash: Hash, vals: Vec<usize>) -> MockCommit {
            MockCommit { hash, vals }
        }
    }
    impl Commit for MockCommit {
        type ValidatorSet = MockValSet;

        fn header_hash(&self) -> Hash {
            self.hash
        }

        // just the intersection
        fn voting_power_in(&self, vals: &Self::ValidatorSet) -> Result<u64, Error> {
            let mut power = 0;
            // if there's a signer thats not in the val set,
            // we can't detect it...
            for signer in self.vals.iter() {
                for val in vals.vals.iter() {
                    if signer == val {
                        power += 1
                    }
                }
            }
            Ok(power)
        }

        fn validate(&self, _vals: &Self::ValidatorSet) -> Result<(), Error> {
            // some implementation specific checks:
            if self.vals.is_empty() || self.hash.algorithm() != Algorithm::Sha256 {
                return Err(ErrorKind::InvalidCommitSignatures {
                    info: "validator set is empty, or, invalid hash algo".to_string(),
                }
                .into());
            }
            Ok(())
        }
    }
    pub type MockSignedHeader = SignedHeader<MockCommit, MockHeader>;
    pub type MockTrustedState = TrustedState<MockCommit, MockHeader>;
    // Mock requester holds a map from height to
    // Headers and commits.
    #[derive(Clone, Debug)]
    pub struct MockRequester {
        pub signed_headers: HashMap<u64, MockSignedHeader>,
        pub validators: HashMap<u64, MockValSet>,
    }
    impl MockRequester {
        pub fn new() -> Self {
            Self {
                signed_headers: HashMap::new(),
                validators: HashMap::new(),
            }
        }
    }
    impl Requester<MockCommit, MockHeader> for MockRequester {
        fn signed_header(&self, h: u64) -> Result<SignedHeader<MockCommit, MockHeader>, ErrorKind> {
            println!("requested signed header for height:{:?}", h);
            if let Some(sh) = self.signed_headers.get(&h) {
                return Ok(sh.to_owned());
            }
            println!("couldn't get sh for: {}", &h);
            Err(ErrorKind::RequestFailed(format!(
                "couldn't get sh for: {}",
                &h
            )))
        }

        fn validator_set(&self, h: u64) -> Result<MockValSet, ErrorKind> {
            println!("requested validators for height:{:?}", h);
            if let Some(vs) = self.validators.get(&h) {
                return Ok(vs.to_owned());
            }
            println!("couldn't get vals for: {}", &h);
            Err(ErrorKind::RequestFailed(format!(
                "couldn't get vals for: {}",
                &h
            )))
        }
    }

    pub fn fixed_hash() -> Hash {
        Hash::new(Algorithm::Sha256, &Sha256::digest(&[5])).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::lite::types::mocks::*;
    use crate::lite::{Commit, Header, SignedHeader, TrustedState, ValidatorSet};
    use crate::lite::{TrustThreshold, TrustThresholdFraction};
    use std::time::SystemTime;

    #[test]
    fn default_is_enough_power() {
        let threshold = TrustThresholdFraction::default();

        // 100% > 33%
        assert!(threshold.is_enough_power(3, 3));

        // 66% > 33%
        assert!(threshold.is_enough_power(2, 3));

        // 33% <= 33%
        assert!(!threshold.is_enough_power(1, 3));

        // 1% < 33%
        assert!(!threshold.is_enough_power(1, 100));
    }

    #[test]
    fn signed_header() {
        let vs = &MockValSet::new(vec![1, 2]);
        let h = MockHeader::new(0, SystemTime::UNIX_EPOCH, vs.hash(), vs.hash());
        let c_header_hash = h.hash();
        let c_vals: Vec<usize> = vec![1];
        let c = MockCommit::new(c_header_hash, c_vals);
        assert_eq!(c.header_hash(), c_header_hash);
        let sh = SignedHeader::new(c.clone(), h.clone());
        assert_eq!(sh.header(), &h);
        assert_eq!(sh.commit(), &c);

        assert_eq!(sh.commit().header_hash(), h.hash());
        assert_eq!(
            sh.commit()
                .voting_power_in(vs)
                .expect("mock shouldn't fail"),
            1
        );
        assert_eq!(sh.header().height(), h.height());
        assert_eq!(sh.header().bft_time(), h.bft_time());
        assert!(sh.commit().validate(vs).is_ok());
    }

    #[test]
    fn trusted_state() {
        let vs = &MockValSet::new(vec![1]);
        let h = MockHeader::new(0, SystemTime::UNIX_EPOCH, vs.hash(), vs.hash());
        let c = MockCommit::new(h.hash(), vec![]);
        let sh = SignedHeader::new(c, h);
        let ts = TrustedState::new(&sh, vs);
        assert_eq!(ts.last_header(), &sh);
        assert_eq!(ts.validators(), vs);
    }

    #[test]
    fn trust_threshold_fraction() {
        assert_eq!(
            TrustThresholdFraction::default(),
            TrustThresholdFraction::new(1, 3).expect("mustn't panic")
        );
        assert!(TrustThresholdFraction::new(2, 3).is_ok());

        assert!(TrustThresholdFraction::new(3, 1).is_err());
        assert!(TrustThresholdFraction::new(1, 0).is_err());
    }
}
