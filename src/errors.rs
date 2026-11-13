//! Error: what stops a run of ghg, each error displayed as the sentence a person reads, from the interface text catalogue.

mod argument_error;
mod build_error;
mod config_error;
mod front_page_error;
mod input_error;
mod input_error_kind;
mod output_error;
mod parse_error;
mod serve_error;

use std::fmt::{Display, Formatter, Result as FmtResult};

use i18n_embed_fl::fl;

pub(crate) use argument_error::ArgumentError;
pub(crate) use build_error::BuildError;
pub(crate) use config_error::{
    AccentShortfall, ConfigError, ConfigErrorKind, ConfigErrors, PatternErrorKind, ValueType,
};
pub(crate) use front_page_error::{FrontPageError, FrontPageErrorKind};
pub(crate) use input_error::{InputError, InputErrors};
pub(crate) use input_error_kind::{InputErrorKind, UnnamedBlock};
pub(crate) use output_error::{OutputError, OutputErrorKind};
pub(crate) use parse_error::ParseError;
pub(crate) use serve_error::{ListenErrorKind, RebuildError, ServeError, WatchError};

use crate::policies::CATALOGUE;

/// `message` said of `path`, and of its line when there is one. The line
/// number is passed as text, so that Fluent never groups its digits.
fn located(f: &mut Formatter<'_>, path: &str, line: Option<usize>, message: &str) -> FmtResult {
    let text = match line {
        Some(line) => fl!(
            CATALOGUE,
            "problem-with-line",
            path = path,
            line = line.to_string(),
            message = message
        ),
        None => fl!(CATALOGUE, "problem", path = path, message = message),
    };
    f.write_str(&text)
}

fn invalid_version_name(name: &str, f: &mut Formatter<'_>) -> FmtResult {
    f.write_str(&fl!(CATALOGUE, "invalid-version-name", name = name))
}

/// Each of `errors` on a line of its own.
fn each_on_a_line<T: Display>(f: &mut Formatter<'_>, errors: &[T]) -> FmtResult {
    errors.iter().try_for_each(|error| writeln!(f, "{error}"))
}
