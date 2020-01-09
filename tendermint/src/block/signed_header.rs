//! SignedHeader contains commit and and block header.
//! It is what the rpc endpoint /commit returns and hence can be used by a
//! light client.
use serde::{Deserialize, Serialize};

use crate::{block, vote::SignedVote};

/// Signed block headers
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignedHeader {
    /// Block header
    pub header: block::Header,
    /// Commit containing signatures for the header
    pub commit: block::Commit,
}

impl SignedHeader {
    /// This is a helper method to iterate over the underlying
    /// votes to compute the voting power (see `voting_power_in` below).
    // TODO move into lite impl module?
    pub fn iter(&self) -> Vec<Option<SignedVote>> {
        let chain_id = self.header.chain_id.to_string();
        let mut votes = self.commit.precommits.clone().into_vec();
        votes
            .drain(..)
            .map(|opt| {
                opt.map(|vote| {
                    SignedVote::new(
                        (&vote).into(),
                        &chain_id,
                        vote.validator_address,
                        vote.signature,
                    )
                })
            })
            .collect()
    }
}
