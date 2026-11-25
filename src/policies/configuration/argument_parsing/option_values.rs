//! Policy: the values of the options that are not plain paths, read as text and checked as clap reads them.

use std::ffi::OsString;

use std::path::PathBuf;

use crate::entities::{Colour, KeptPattern, MessageFormat, PathList, Theme, VersionName};
use crate::errors::AccentShortfall;
use crate::policies::configuration::check_accent;

/// Why a value was refused, which clap keeps as the source of its error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(super) enum Rejected {
    #[error("the value is empty")]
    Empty,
    #[error("the value is not a port")]
    NotAPort,
    #[error("the value is not a version name")]
    NotAVersionName,
    /// `reason` is the `ignore` crate's.
    #[error("the value is not a pattern: {reason}")]
    NotAPattern { reason: String },
    #[error("the value is not a message format")]
    NotAMessageFormat,
    #[error("the value is not a colour")]
    NotAColour,
    #[error("the accent has too little contrast")]
    TooLittleContrast { shortfall: AccentShortfall },
}

/// A value as text, with what is not Unicode replaced. An empty value
/// names nothing, and would be reported as nothing.
pub(super) fn text(value: OsString) -> Result<String, Rejected> {
    let text = value.to_string_lossy().into_owned();
    if text.is_empty() {
        return Err(Rejected::Empty);
    }
    Ok(text)
}

/// Only digits, so that `+80` or ` 80` is not taken for 80.
pub(super) fn port(value: OsString) -> Result<u16, Rejected> {
    let value = text(value)?;
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit())
        .then(|| value.parse().ok())
        .flatten()
        .ok_or(Rejected::NotAPort)
}

pub(super) fn version_name(value: OsString) -> Result<VersionName, Rejected> {
    VersionName::new(&text(value)?).ok_or(Rejected::NotAVersionName)
}

pub(super) fn kept_pattern(value: OsString) -> Result<KeptPattern, Rejected> {
    KeptPattern::new(&text(value)?).map_err(|reason| Rejected::NotAPattern { reason })
}

/// `-` for standard input, and any other value for the file of that path.
pub(super) fn path_list(value: OsString) -> Result<PathList, Rejected> {
    if value.is_empty() {
        return Err(Rejected::Empty);
    }
    if value == "-" {
        return Ok(PathList::StandardInput);
    }
    Ok(PathList::File(PathBuf::from(value)))
}

/// `human` or `json`, as written, in lower case.
pub(super) fn message_format(value: OsString) -> Result<MessageFormat, Rejected> {
    match text(value)?.as_str() {
        "human" => Ok(MessageFormat::Human),
        "json" => Ok(MessageFormat::Json),
        _ => Err(Rejected::NotAMessageFormat),
    }
}

pub(super) fn light_accent(value: OsString) -> Result<Colour, Rejected> {
    accent(Theme::Light, value)
}

pub(super) fn dark_accent(value: OsString) -> Result<Colour, Rejected> {
    accent(Theme::Dark, value)
}

/// A colour in hexadecimal, as `accent_light` and `accent_dark` take it, that reaches the
/// contrast `theme` needs.
fn accent(theme: Theme, value: OsString) -> Result<Colour, Rejected> {
    let colour = Colour::from_hex(&text(value)?).ok_or(Rejected::NotAColour)?;
    match check_accent(theme, colour) {
        Some(shortfall) => Err(Rejected::TooLittleContrast { shortfall }),
        None => Ok(colour),
    }
}
