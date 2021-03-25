//! Provides [proptest](https://github.com/AltSysrq/proptest) generators for
//! tendermint validator.

use proptest::prelude::*;
use std::convert::TryFrom;
use tendermint::account::Id;

prop_compose! {
    pub fn arb_id()
    (a in prop::array::uniform20(0u8..))
    -> Id {
        Id::try_from(a.to_vec()).unwrap()
    }
}
