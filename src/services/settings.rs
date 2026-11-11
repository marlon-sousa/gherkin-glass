//! Service: finding how a build is run, from the command line's options and the configuration file.

use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;

use crate::entities::{BuildOptions, Config, ConfigProblem, ConfigProblemKind, Settings};
use crate::policies::{config_folder, parse_config, resolve_settings};
use crate::ports::ConfigReader;

/// The configuration file looked for in the current folder.
const CONFIG_FILE: &str = "ghg.toml";

pub(crate) struct SettingsService {
    reader: Arc<dyn ConfigReader>,
}

/// A configuration file that stops the run: the file as it was given, and
/// every problem in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConfigProblems {
    pub(crate) file: PathBuf,
    pub(crate) problems: Vec<ConfigProblem>,
}

impl SettingsService {
    pub(crate) fn new(reader: Arc<dyn ConfigReader>) -> Self {
        SettingsService { reader }
    }

    pub(crate) fn load(&self, options: &BuildOptions) -> Result<Settings, ConfigProblems> {
        let asked_for = options.config.is_some();
        let file = options
            .config
            .clone()
            .unwrap_or_else(|| PathBuf::from(CONFIG_FILE));
        let config = match self.reader.read(&file) {
            Ok(source) => parse_config(&source),
            // Only a file that was not asked for may be missing.
            Err(error) if !asked_for && error.kind() == ErrorKind::NotFound => {
                Ok(Config::default())
            }
            Err(error) => Err(vec![ConfigProblem {
                line: None,
                kind: ConfigProblemKind::CannotBeRead {
                    reason: error.to_string(),
                },
            }]),
        };
        match config {
            Ok(config) => Ok(resolve_settings(options, &config_folder(&file), &config)),
            Err(problems) => Err(ConfigProblems { file, problems }),
        }
    }
}
