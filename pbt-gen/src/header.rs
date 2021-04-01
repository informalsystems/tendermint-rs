use crate::{height::arb_height, time::arb_datetime};
use proptest::prelude::*;
use tendermint::block::header::Header;

prop_compose! {
    pub fn fuzz_header(h: Header)
    (
        datetime in arb_datetime(),
        height in arb_height(),
    )
    -> Header {
        let mut header: Header = h.clone();
        header.time = datetime.into();
        header.height = height;
        header
    }
}
