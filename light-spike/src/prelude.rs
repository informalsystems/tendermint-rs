pub use crate::operations::*;
pub use crate::trace::*;
pub use crate::types::*;

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
