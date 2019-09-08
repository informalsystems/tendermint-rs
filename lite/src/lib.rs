/// TrustedState stores the latest state trusted by a lite client,
/// including the last header and the validator set to use to verify
/// the next header.
struct TrustedState<H, V>
where
    H: Header,
    V: Validators,
{
    header: H,
    next_validators: V,
}

/// Need to do something better here :)
type Height = u64;
type Hash = u64; // TODO
type Time = u64; // TODO
type BlockID = u64; // TODO
type Bytes = u64; // TODO

/// Header contains meta data about the block -
/// the height, the time, and the hash of the validator set
/// that should sign this header, and the hash of the validator
/// set that should sign the next header.
trait Header {
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
trait Validators {
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
trait Validator {
    fn id(&self) -> Hash;
    fn power(&self) -> u64;
    fn verify(&self, sign_bytes: Bytes, signature: Bytes) -> bool;
}

/// Commit is proof a Header is valid.
/// It has an underlying Vote type with the relevant vote data
/// for verification.
trait Commit {
    type Vote: Vote;

    /// Hash of the header this commit is for.
    fn header_hash(&self) -> Hash;

    /// Actual block_id this commit is for.
    /// NOTE: in tendermint, the header_hash is a field in the BlockID
    fn block_id(&self) -> BlockID;

    /// Return the underlying votes for iteration.
    fn into_vec(&self) -> Vec<Self::Vote>;
}

/// Vote is the vote for a validator on some block_id.
/// Note it does not expose height/round information, but
/// is expected to have implicit access to it so it can produce
/// correct sign_bytes.
trait Vote {
    fn block_id(&self) -> BlockID;
    fn sign_bytes(&self) -> Bytes;
    fn signature(&self) -> Bytes;
}

/// The sequentially verifying lite client
/// verifies all headers in order, where +2/3 of the correct
/// validator set must have signed the header.
struct SequentialVerifier<H, V>
where
    H: Header,
    V: Validators,
{
    trusting_period: Time,
    state: TrustedState<H, V>,
}

impl<H, V> SequentialVerifier<H, V>
where
    H: Header,
    V: Validators,
{
    /// trusted state expires after the trusting period.
    fn expires(&self) -> Time {
        self.state.header.bft_time() + self.trusting_period
    }

    /// Verify takes a header, a commit for the header, and the next validator set referenced by
    /// the header. Without knowing this next validator set, we can't really verify the next
    /// header, so we make verifying this header conditional on receiving that validator set.
    /// Returns the new TrustedState if verification passes.
    fn verify<C>(
        self,
        now: Time,
        header: H,
        commit: C,
        next_validators: V,
    ) -> Option<TrustedState<H, V>>
    where
        C: Commit,
    {
        // check if the state expired
        if self.expires() < now {
            return None;
        }

        // sequeuntial height only
        if header.height() != self.state.header.height() + 1 {
            return None;
        }

        // validator set for this header is already trusted
        if header.validators_hash() != self.state.next_validators.hash() {
            return None;
        }

        // next validator set to trust is correctly supplied
        if header.next_validators_hash() != next_validators.hash() {
            return None;
        }

        // commit is for a block with this header
        if header.hash() != commit.header_hash() {
            return None;
        }

        // check that +2/3 validators signed the block_id
        if self.verify_commit_full(commit) {
            return None;
        }

        Some(TrustedState {
            header,
            next_validators,
        })
    }

    /// Check that +2/3 of the trusted validator set signed this commit.
    fn verify_commit_full<C>(self, commit: C) -> bool
    where
        C: Commit,
    {
        let mut signed_power: u64 = 0;
        let mut total_power: u64 = 0;

        let vals_iter = self.state.next_validators.into_vec().into_iter();
        let commit_iter = commit.into_vec().into_iter();

        for (val, vote) in vals_iter.zip(commit_iter) {
            total_power += val.power();

            // skip if vote is not for the right block id
            if vote.block_id() != commit.block_id() {
                continue;
            }

            // check vote is valid from validator
            if !val.verify(vote.sign_bytes(), vote.signature()) {
                return false;
            }
            signed_power += val.power();
        }
        signed_power * 3 > total_power * 2
    }
}
