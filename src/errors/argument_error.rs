//! Error: a mistake on the command line.

use std::fmt::{Formatter, Result as FmtResult};

use i18n_embed_fl::fl;
use thiserror::Error;

use super::invalid_version_name;
use crate::policies::CATALOGUE;

/// Each `String` is what was typed, shown as it was typed, an option with its dashes.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum ArgumentError {
    #[error(fmt = Self::missing_command)]
    MissingCommand,
    #[error(fmt = Self::unknown_command)]
    UnknownCommand { name: String },
    /// `command` is `ghg`, `ghg build` or `ghg serve`.
    #[error(fmt = Self::unknown_option)]
    UnknownOption { command: String, option: String },
    #[error(fmt = Self::missing_value)]
    MissingValue { option: String },
    #[error(fmt = Self::unexpected_value)]
    UnexpectedValue { option: String },
    #[error(fmt = Self::repeated_option)]
    RepeatedOption { option: String },
    #[error(fmt = Self::unexpected_argument)]
    UnexpectedArgument { command: String, argument: String },
    #[error(fmt = Self::not_a_port)]
    NotAPort { value: String },
    #[error(fmt = Self::watch_without_build)]
    WatchWithoutBuild,
    #[error(fmt = invalid_version_name)]
    InvalidVersionName { name: String },
}

impl ArgumentError {
    fn missing_command(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "missing-command"))
    }

    fn unknown_command(name: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "unknown-command", name = name))
    }

    fn unknown_option(command: &str, option: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(
            CATALOGUE,
            "unknown-option",
            command = command,
            option = option
        ))
    }

    fn missing_value(option: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "missing-value", option = option))
    }

    fn unexpected_value(option: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "unexpected-value", option = option))
    }

    fn repeated_option(option: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "repeated-option", option = option))
    }

    fn unexpected_argument(command: &str, argument: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(
            CATALOGUE,
            "unexpected-argument",
            command = command,
            argument = argument
        ))
    }

    fn not_a_port(value: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "not-a-port", value = value))
    }

    fn watch_without_build(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "watch-without-build"))
    }
}
