//! Re-exports all the types which are commonly used within the light client codebase.

pub use std::time::{Duration, SystemTime};

pub use crate::{bail, ensure};
pub use crate::{
    components::{clock::*, io::*, scheduler::*, verifier::*},
    errors::*,
    light_client::*,
    operations::*,
    predicates::{errors::*, ProdPredicates, VerificationPredicates},
    state::*,
    store::{memory::*, sled::*, LightStore, VerifiedStatus},
    types::*,
};

pub fn todo<A>() -> A {
    unreachable!()
}
