//! Abscissa `Application` for the KMS

use abscissa::{Application, LoggingConfig};

use commands::KmsCommand;
use config::KmsConfig;

#[derive(Debug)]
pub struct KmsApplication;

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
