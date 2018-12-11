//! Abscissa `Application` for the KMS

use abscissa::{self, Application, LoggingConfig};

use crate::{commands::KmsCommand, config::KmsConfig};

/// The `tmkms` application
#[derive(Debug)]
pub struct KmsApplication;

impl KmsApplication {
    /// Boot the application
    // TODO: use the upstream implementation of this method
    pub fn boot() {
        abscissa::boot(KmsApplication)
    }
}

impl Application for KmsApplication {
    type Cmd = KmsCommand;
    type Config = KmsConfig;

    fn logging_config(&self, command: &KmsCommand) -> LoggingConfig {
        if command.verbose() {
            LoggingConfig::verbose()
        } else {
            LoggingConfig::default()
        }
    }
}
