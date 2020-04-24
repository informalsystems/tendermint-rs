pub use crate::components::rpc::Rpc;
pub use crate::components::scheduler::Scheduler;
pub use crate::components::verifier::Verifier;
pub use crate::errors::*;
pub use crate::event::*;
pub use crate::operations::*;
pub use crate::predicates::VerificationPredicates;
pub use crate::trace::*;
pub use crate::trusted_store::*;
pub use crate::types::*;
pub use crate::{ensure, impl_event};

pub use std::time::{Duration, SystemTime};

pub(crate) trait BoolExt {
    fn true_or<E>(self, e: E) -> Result<(), E>;
    fn false_or<E>(self, e: E) -> Result<(), E>;
}

impl BoolExt for bool {
    fn true_or<E>(self, e: E) -> Result<(), E> {
        if self {
            Ok(())
        } else {
            Err(e)
        }
    }

    fn false_or<E>(self, e: E) -> Result<(), E> {
        if !self {
            Ok(())
        } else {
            Err(e)
        }
    }
}
