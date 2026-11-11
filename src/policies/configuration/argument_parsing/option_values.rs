//! Policy: the values of the options that are not paths, read as text and checked as clap reads them.

use std::ffi::OsString;

use crate::entities::VersionName;

/// Why a value was refused, which clap keeps as the source of its error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(super) enum Rejected {
    #[error("the value is empty")]
    Empty,
    #[error("the value is not a port")]
    NotAPort,
    #[error("the value is not a version name")]
    NotAVersionName,
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
