use tendermint::lite::TrustedState;
use tendermint::{block::signed_header::SignedHeader, validator::Set};

#[derive(Clone)]
pub struct State {
    last_header: SignedHeader,
    vals: Set,
}

impl TrustedState for State {
    type LastHeader = SignedHeader;
    type ValidatorSet = Set;

    fn new(last_header: &Self::LastHeader, vals: &Self::ValidatorSet) -> Self {
        State {
            last_header: last_header.clone(),
            vals: vals.clone(),
        }
    }

    // height H-1
    fn last_header(&self) -> &Self::LastHeader {
        &self.last_header
    }

    // height H
    fn validators(&self) -> &Self::ValidatorSet {
        &self.vals
    }
}
