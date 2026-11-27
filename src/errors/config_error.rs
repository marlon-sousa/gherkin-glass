//! Error: what is wrong with a configuration file, each error a thing that stops the run.

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;
use std::path::PathBuf;

use i18n_embed_fl::fl;
use thiserror::Error;

use super::{
    LocatedError, each_on_a_line, invalid_kept_pattern, not_boolean, not_colour, not_regex,
    regex_too_large, too_little_contrast, unknown_group,
};
use crate::entities::{Colour, Theme};
use crate::policies::CATALOGUE;

/// A configuration file that stops the run: the file as it was given, and
/// every error in it. Its display is the report, one line per error.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) struct ConfigErrors {
    pub(crate) file: PathBuf,
    pub(crate) errors: Vec<ConfigError>,
}

/// One error, on its line of the file when it has one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ConfigError {
    pub(crate) line: Option<usize>,
    pub(crate) kind: ConfigErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
pub(crate) enum ConfigErrorKind {
    /// `reason` is the operating system's.
    #[error(fmt = Self::cannot_be_read)]
    CannotBeRead { reason: String },
    /// Not TOML; `message` is the `toml` crate's, passed through as it is.
    #[error("{message}")]
    Syntax { message: String },
    #[error(fmt = Self::unknown_key)]
    UnknownKey { key: String },
    #[error(fmt = Self::empty)]
    Empty { key: String },
    #[error(fmt = Self::wrong_type)]
    WrongType { key: String, expected: ValueType },
    #[error(fmt = too_little_contrast)]
    TooLittleContrast {
        key: String,
        shortfall: AccentShortfall,
    },
    #[error(fmt = Self::not_table_list)]
    NotTableList,
    /// `keep_in_output`, not a list of text.
    #[error(fmt = Self::not_text_list)]
    NotTextList,
    /// `reason` is the `ignore` crate's.
    #[error(fmt = invalid_kept_pattern)]
    InvalidKeptPattern { pattern: String, reason: String },
    /// A key a table of `tag_links` needs.
    #[error(fmt = Self::missing)]
    Missing { key: String },
    /// `reason` is the `regex-syntax` crate's.
    #[error(fmt = not_regex)]
    NotRegex { key: String, reason: String },
    #[error(fmt = regex_too_large)]
    RegexTooLarge { key: String },
    /// `group` is a number or a name, as the replacement writes it.
    #[error(fmt = unknown_group)]
    UnknownGroup { key: String, group: String },
}

/// An accent colour that does not reach the contrast its theme requires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AccentShortfall {
    pub(crate) theme: Theme,
    pub(crate) accent: Colour,
    /// The theme's background the accent contrasts least with.
    pub(crate) background: Colour,
    /// The ratio it reaches against that background, in hundredths, rounded down.
    pub(crate) hundredths: u32,
}

/// The kinds of value a key takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ValueType {
    Text,
    Boolean,
    /// Text naming a colour, such as `"#0b4f9c"`.
    Colour,
}

/// Why a pattern of `tag_links` cannot be used, before it is said of its key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PatternErrorKind {
    /// The `regex-syntax` crate's short description, such as `unclosed group`.
    Syntax(String),
    /// Over the size limit the `regex` crate sets too.
    TooLarge,
}

impl Display for ConfigErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        each_on_a_line(f, &self.problems())
    }
}

impl ConfigErrors {
    /// Each error as its line of the report, said of the file as it was given.
    pub(crate) fn problems(&self) -> Vec<LocatedError> {
        let path = self.file.to_string_lossy();
        self.errors
            .iter()
            .map(|error| LocatedError {
                path: path.to_string(),
                line: error.line,
                message: error.kind.to_string(),
            })
            .collect()
    }

    /// The report of `file`, named as it was given, when it cannot be read.
    pub(crate) fn unreadable(file: PathBuf, error: &IoError) -> Self {
        ConfigErrors {
            file,
            errors: vec![ConfigError::cannot_be_read(error)],
        }
    }
}

impl ConfigError {
    /// The error of a file that cannot be read, in the operating system's words.
    pub(crate) fn cannot_be_read(error: &IoError) -> Self {
        ConfigError {
            line: None,
            kind: ConfigErrorKind::CannotBeRead {
                reason: error.to_string(),
            },
        }
    }
}

impl ConfigErrorKind {
    fn cannot_be_read(reason: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "config-cannot-be-read", reason = reason))
    }

    fn unknown_key(key: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "config-unknown-key", key = key))
    }

    fn empty(key: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "config-empty", key = key))
    }

    fn wrong_type(key: &str, expected: &ValueType, f: &mut Formatter<'_>) -> FmtResult {
        match expected {
            ValueType::Text => f.write_str(&fl!(CATALOGUE, "config-not-text", key = key)),
            ValueType::Boolean => not_boolean(key, f),
            ValueType::Colour => not_colour(key, f),
        }
    }

    fn not_table_list(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "config-not-table-list"))
    }

    fn not_text_list(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "config-not-text-list"))
    }

    fn missing(key: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "config-missing", key = key))
    }
}
