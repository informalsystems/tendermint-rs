//! Abscissa `Application` for the KMS

use abscissa::{Application, LoggingConfig};

use commands::KMSCommand;
use config::KMSConfig;

#[derive(Debug)]
pub struct KMSApplication;

impl Application for KMSApplication {
    type Cmd = KMSCommand;
    type Config = KMSConfig;

    fn logging_config(&self, command: &KMSCommand) -> LoggingConfig {
        if command.verbose() {
            LoggingConfig::verbose()
        } else {
            LoggingConfig::default()
        }
    }
}
