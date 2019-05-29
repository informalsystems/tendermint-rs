//! Application-local prelude: conveniently import types/functions/macros
//! which are generally useful and should be available everywhere.

/// Application state accessors
pub use crate::application::{app_config, app_reader, app_writer};

/// Commonly used Abscissa traits
pub use abscissa::{Application, Command, Runnable};

/// Logging macros
pub use log::{debug, error, info, log, log_enabled, trace, warn};
