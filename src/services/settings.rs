//! Service: finding how a build is run, from the command line's options, the configuration file and the list of feature files `--only` names.

use std::io::{Error as IoError, ErrorKind};
use std::path::PathBuf;
use std::sync::Arc;

use crate::entities::{BuildOptions, Config, Destination, ListedFiles, PathList, Settings};
use crate::errors::{ConfigError, ConfigErrorKind, ConfigErrors};
use crate::policies::{config_folder, listed_paths, parse_config, resolve_settings};
use crate::ports::{FileReader, StandardInput};

/// The configuration file looked for in the current folder.
const CONFIG_FILE: &str = "ghg.toml";

/// What the controller asks of the settings service.
pub(crate) trait SettingsApi: Send + Sync {
    /// The configuration file `options` name, or the one looked for in the current folder.
    fn config_file(&self, options: &BuildOptions) -> PathBuf;

    /// The settings of a build into `destination`.
    fn load(
        &self,
        options: &BuildOptions,
        destination: Destination,
    ) -> Result<Settings, ConfigErrors>;
}

#[derive(Clone)]
pub(crate) struct SettingsService {
    reader: Arc<dyn FileReader>,
    standard_input: Arc<dyn StandardInput>,
}

impl SettingsService {
    pub(crate) fn new(reader: Arc<dyn FileReader>, standard_input: Arc<dyn StandardInput>) -> Self {
        SettingsService {
            reader,
            standard_input,
        }
    }

    /// The paths `list` holds, or the error that stops the run when it cannot be read.
    fn listed_files(&self, list: &PathList) -> Result<ListedFiles, ConfigErrors> {
        let text = match list {
            PathList::File(file) => self.reader.read(file),
            PathList::StandardInput => self.standard_input.read(),
        };
        text.map(|text| listed(list, &text))
            .map_err(|error| unreadable_list(list, &error))
    }
}

impl SettingsApi for SettingsService {
    fn config_file(&self, options: &BuildOptions) -> PathBuf {
        options
            .config
            .clone()
            .unwrap_or_else(|| PathBuf::from(CONFIG_FILE))
    }

    fn load(
        &self,
        options: &BuildOptions,
        destination: Destination,
    ) -> Result<Settings, ConfigErrors> {
        let asked_for = options.config.is_some();
        let file = self.config_file(options);
        let config = match self.reader.read(&file) {
            Ok(source) => parse_config(&source),
            // Only a file that was not asked for may be missing.
            Err(error) if !asked_for && error.kind() == ErrorKind::NotFound => {
                Ok(Config::default())
            }
            Err(error) => Err(vec![cannot_be_read(&error)]),
        };
        let config = match config {
            Ok(config) => config,
            Err(errors) => return Err(ConfigErrors { file, errors }),
        };
        let only = options
            .only
            .as_ref()
            .map(|list| self.listed_files(list))
            .transpose()?;
        Ok(resolve_settings(
            options,
            &config_folder(&file),
            &config,
            destination,
            only,
        ))
    }
}

/// The paths `text`, the list read from `list`, holds.
fn listed(list: &PathList, text: &str) -> ListedFiles {
    ListedFiles {
        list: list.clone(),
        paths: listed_paths(text),
    }
}

/// The error of `list` when it cannot be read, named as it was given.
fn unreadable_list(list: &PathList, error: &IoError) -> ConfigErrors {
    ConfigErrors {
        file: list.as_given(),
        errors: vec![cannot_be_read(error)],
    }
}

/// The error of a file that cannot be read, in the operating system's words.
fn cannot_be_read(error: &IoError) -> ConfigError {
    ConfigError {
        line: None,
        kind: ConfigErrorKind::CannotBeRead {
            reason: error.to_string(),
        },
    }
}

#[cfg(test)]
mod tests;
