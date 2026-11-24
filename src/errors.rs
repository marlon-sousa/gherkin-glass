//! Error: what stops a run of ghg, each error displayed as the sentence a person reads, from the interface text catalogue.

mod argument_error;
mod build_error;
mod config_error;
mod front_page_error;
mod input_error;
mod input_error_kind;
mod located_error;
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
pub(crate) use located_error::LocatedError;
pub(crate) use output_error::{OutputError, OutputErrorKind};
pub(crate) use parse_error::ParseError;
pub(crate) use serve_error::{ListenErrorKind, RebuildError, ServeError, SignalsError, WatchError};

use crate::policies::CATALOGUE;

fn invalid_version_name(name: &str, f: &mut Formatter<'_>) -> FmtResult {
    f.write_str(&fl!(CATALOGUE, "invalid-version-name", name = name))
}

fn invalid_kept_pattern(pattern: &str, reason: &str, f: &mut Formatter<'_>) -> FmtResult {
    f.write_str(&fl!(
        CATALOGUE,
        "invalid-kept-pattern",
        pattern = pattern,
        reason = reason
    ))
}

/// Each of `errors` on a line of its own.
fn each_on_a_line<T: Display>(f: &mut Formatter<'_>, errors: &[T]) -> FmtResult {
    errors.iter().try_for_each(|error| writeln!(f, "{error}"))
}
