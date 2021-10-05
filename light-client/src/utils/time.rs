use std::time::Duration;

use flex_error::define_error;

#[cfg(feature = "rpc-client")]
use futures::future::{FusedFuture, FutureExt as _};
#[cfg(feature = "rpc-client")]
use futures_timer::Delay;

define_error! {
    #[derive(Debug)]
    TimeError {
        Timeout
            { duration: Duration }
            | e | {
                format_args!("task timed out after {} ms",
                    e.duration.as_millis())
            },
    }
}

impl TimeErrorDetail {
    /// Whether this error means that a timeout occured when querying a node.
    pub fn is_timeout(&self) -> Option<Duration> {
        match self {
            Self::Timeout(e) => Some(e.duration),
        }
    }
}

#[cfg(feature = "rpc-client")]
pub async fn timeout<T>(duration: Duration, task: T) -> Result<T::Output, TimeError>
where
    T: FusedFuture,
{
    let mut task = Box::pin(task);
    let mut delay = Delay::new(duration).fuse();
    futures::select! {
        out = task => Ok(out),
        _ = delay => Err(TimeError::timeout(duration)),
    }
}
