struct TrustedState<H, V>
where
    H: Header,
    V: Validators,
{
    header: H,
    next_validators: V,
}

type Height = u64;
type Hash = u64; // TODO
type Time = u64; // TODO
type BlockID = u64; // TODO
type Bytes = u64; // TODO

trait Header {
    fn height(&self) -> Height;
    fn bft_time(&self) -> Time;
    fn validators_hash(&self) -> Hash;
    fn next_validators_hash(&self) -> Hash;

    fn hash(&self) -> Hash;
}

trait Validators {
    fn hash(&self) -> Hash;
    fn vals(&self) -> Vec<Validator>;

    fn verify(&self, val_id: Hash, sign_bytes: Bytes, signature: Bytes) -> bool;
}

struct Validator {
    id: Hash,
    power: u64
}

trait Commit<V> where V: Vote{
    fn header_hash(&self) -> Hash;
    fn block_id(&self) -> BlockID;
    fn votes(&self) -> Vec<V>;
}

trait Vote {
    fn block_id(&self) -> BlockID;
    fn sign_bytes(&self) -> Bytes;
    fn signature(&self) -> Bytes;
}

struct SequentialVerifier<H, V> 
where 
    H: Header, 
    V: Validators,
{
    trusting_period: Time,
    state: TrustedState<H,V>,
}

impl <H,V> SequentialVerifier<H,V> 
where
    H: Header,
    V: Validators,
{
    fn expires(&self) -> Time {
        self.state.header.bft_time() + self.trusting_period
    }

    // returns the new trusted state if it verifies, else None
    fn verify<C, VOTE>(self, now: Time, header: H, commit: C, next_validators: V) -> Option<TrustedState<H,V>> 
        where C: Commit<VOTE>,
              VOTE: Vote,
    {
        // check if the state expired
        if self.expires() < now {
            return None
        }

        // sequeuntial height only
        if header.height() != self.state.header.height() + 1 {
            return None
        }
       
        // validator set for this header is already trusted
        if header.validators_hash() != self.state.next_validators.hash() {
            return None
        }

        
        // next validator set to trust is correctly supplied
        if header.next_validators_hash() != next_validators.hash() {
             return None
        }

        
        // commit is for a block with this header
        if header.hash() != commit.header_hash() {
             return None
        }

         
        // check that +2/3 validators signed the block_id
        if self.verify_commit_full(commit) {
             return None
        }

        Some( TrustedState{ header, next_validators } )
    }

    fn verify_commit_full<C, VOTE>(self, commit: C) -> bool
    where
        C: Commit<VOTE>,
        VOTE: Vote,
    {

        let mut signed_power: u64 = 0;
        let mut total_power: u64 = 0;

        let vals = self.state.next_validators;
        let vals_iter = vals.vals().into_iter();
        let commit_iter = commit.votes().into_iter();
        

        for (val, vote) in vals_iter.zip(commit_iter) {
            total_power += val.power;

            // skip if vote is not for the right block id
            if vote.block_id() != commit.block_id() {
                continue
            }

            // check vote is valid from validator
            let sign_bytes = vote.sign_bytes();
            let signature = vote.signature();
            if !vals.verify(val.id, sign_bytes, signature) {
                return false
            }
            signed_power += val.power;

        }
        signed_power * 3 > total_power * 2
    }
}
