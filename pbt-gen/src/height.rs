//! Provides [proptest](https://github.com/AltSysrq/proptest) generators for
//! tendermint height.

use proptest::prelude::*;
use std::convert::TryFrom;
use tendermint::block::Height;

prop_compose! {
    pub fn arb_height()
        (
        // for now changing this to i64
        // TODO: ideally it should be u64
        // see: https://github.com/informalsystems/tendermint-rs/issues/830
            h in 0..i64::MAX
        ) -> Height {
            let ret = Height::try_from(h);
            ret.unwrap()
        }
}
