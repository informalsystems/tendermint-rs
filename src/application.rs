//! Abscissa `Application` for the KMS

use crate::{commands::KmsCommand, config::KmsConfig};
use abscissa::{application, logging, Application, FrameworkError, StandardPaths};
use lazy_static::lazy_static;

lazy_static! {
    /// Application state
    pub static ref APPLICATION: application::Lock<KmsApplication> = application::Lock::default();
}

/// Obtain a read-only (multi-reader) lock on the application state.
///
/// Panics if the application state has not been initialized.
pub fn app_reader() -> application::lock::Reader<KmsApplication> {
    APPLICATION.read()
}

/// Obtain an exclusive mutable lock on the application state.
pub fn app_writer() -> application::lock::Writer<KmsApplication> {
    APPLICATION.write()
}

/// Obtain a read-only (multi-reader) lock on the application configuration.
///
/// Panics if the application configuration has not been loaded.
pub fn app_config() -> abscissa::config::Reader<KmsApplication> {
    abscissa::config::Reader::new(&APPLICATION)
}

/// The `tmkms` application
#[derive(Debug)]
pub struct KmsApplication {
    /// Application configuration.
    config: Option<KmsConfig>,

    /// Application state.
    state: application::State<Self>,
}

impl Default for KmsApplication {
    fn default() -> Self {
        Self {
            config: None,
            state: application::State::default(),
        }
    }
}

impl Application for KmsApplication {
    /// Entrypoint command for this application.
    type Cmd = KmsCommand;

    /// Application configuration.
    type Cfg = KmsConfig;

    /// Paths to resources within the application.
    type Paths = StandardPaths;

    /// Accessor for application configuration.
    fn config(&self) -> Option<&KmsConfig> {
        self.config.as_ref()
    }

    /// Borrow the application state immutably.
    fn state(&self) -> &application::State<Self> {
        &self.state
    }

    /// Borrow the application state mutably.
    fn state_mut(&mut self) -> &mut application::State<Self> {
        &mut self.state
    }

    /// Register all components used by this application.
    ///
    /// If you would like to add additional components to your application
    /// beyond the default ones provided by the framework, this is the place
    /// to do so.
    fn register_components(&mut self, command: &Self::Cmd) -> Result<(), FrameworkError> {
        let components = self.framework_components(command)?;
        self.state.components.register(components)
    }

    /// Post-configuration lifecycle callback.
    ///
    /// Called regardless of whether config is loaded to indicate this is the
    /// time in app lifecycle when configuration would be loaded if
    /// possible.
    fn after_config(&mut self, config: Option<Self::Cfg>) -> Result<(), FrameworkError> {
        // Provide configuration to all component `after_config()` handlers
        for component in self.state.components.iter_mut() {
            component.after_config(config.as_ref())?;
        }

        self.config = config;
        Ok(())
    }

    /// Get logging configuration from command-line options
    fn logging_config(&self, command: &KmsCommand) -> logging::Config {
        if command.verbose() {
            logging::Config::verbose()
        } else {
            logging::Config::default()
        }
    }
}
