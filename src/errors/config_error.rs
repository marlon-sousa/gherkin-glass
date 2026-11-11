//! Error: what is wrong with a configuration file, each error a thing that stops the run.

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::PathBuf;

use i18n_embed_fl::fl;
use thiserror::Error;

use super::{invalid_version_name, located};
use crate::entities::{AccentShortfall, Theme};
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
    /// `name` is the text `version_name` was given.
    #[error(fmt = invalid_version_name)]
    InvalidVersionName { name: String },
    #[error(fmt = Self::wrong_type)]
    WrongType { key: String, expected: ValueType },
    #[error(fmt = Self::too_little_contrast)]
    TooLittleContrast {
        key: String,
        shortfall: AccentShortfall,
    },
    #[error(fmt = Self::not_table_list)]
    NotTableList,
    /// A key a table of `tag_links` needs.
    #[error(fmt = Self::missing)]
    Missing { key: String },
    /// `reason` is the `regex-syntax` crate's.
    #[error(fmt = Self::not_regex)]
    NotRegex { key: String, reason: String },
    #[error(fmt = Self::regex_too_large)]
    RegexTooLarge { key: String },
    /// `group` is a number or a name, as the replacement writes it.
    #[error(fmt = Self::unknown_group)]
    UnknownGroup { key: String, group: String },
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
        let path = self.file.to_string_lossy();
        self.errors
            .iter()
            .try_for_each(|error| error.report(&path, f))
    }
}

impl ConfigError {
    /// The error said of `path`, the file it is in, on a line of its own.
    fn report(&self, path: &str, f: &mut Formatter<'_>) -> FmtResult {
        located(f, path, self.line, &self.kind.to_string())?;
        writeln!(f)
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
        f.write_str(&match expected {
            ValueType::Text => fl!(CATALOGUE, "config-not-text", key = key),
            ValueType::Boolean => fl!(CATALOGUE, "config-not-boolean", key = key),
            ValueType::Colour => fl!(CATALOGUE, "config-not-colour", key = key),
        })
    }

    /// The colours and the ratio are passed as text, so that Fluent never formats them.
    fn too_little_contrast(
        key: &str,
        shortfall: &AccentShortfall,
        f: &mut Formatter<'_>,
    ) -> FmtResult {
        let colour = shortfall.accent.to_string();
        let background = shortfall.background.to_string();
        let ratio = format!(
            "{}.{:02}",
            shortfall.hundredths / 100,
            shortfall.hundredths % 100
        );
        f.write_str(&match shortfall.theme {
            Theme::Light => fl!(
                CATALOGUE,
                "config-accent-light-contrast",
                key = key,
                colour = colour,
                ratio = ratio,
                background = background
            ),
            Theme::Dark => fl!(
                CATALOGUE,
                "config-accent-dark-contrast",
                key = key,
                colour = colour,
                ratio = ratio,
                background = background
            ),
        })
    }

    fn not_table_list(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "config-not-table-list"))
    }

    fn missing(key: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "config-missing", key = key))
    }

    fn not_regex(key: &str, reason: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(
            CATALOGUE,
            "config-not-regex",
            key = key,
            reason = reason
        ))
    }

    fn regex_too_large(key: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "config-regex-too-large", key = key))
    }

    fn unknown_group(key: &str, group: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(
            CATALOGUE,
            "config-unknown-group",
            key = key,
            group = group
        ))
    }
}
