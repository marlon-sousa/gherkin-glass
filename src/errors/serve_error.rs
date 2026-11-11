//! Error: what stops ghg serve before it serves, and what stops a rebuild while it serves.

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::PathBuf;

use i18n_embed_fl::fl;
use thiserror::Error;

use super::{ConfigErrors, InputErrors, OutputError};
use crate::policies::CATALOGUE;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum ServeError {
    #[error(transparent)]
    Config(ConfigErrors),
    #[error(transparent)]
    Input(InputErrors),
    #[error(transparent)]
    Output(OutputError),
    #[error(fmt = Self::cannot_listen)]
    Listen {
        host: String,
        port: u16,
        kind: ListenErrorKind,
    },
    #[error(transparent)]
    Watch(WatchError),
}

/// Why ghg serve cannot listen, said of its host and port by [`ServeError::Listen`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ListenErrorKind {
    PortTaken,
    /// The reason is the operating system's words.
    Other {
        reason: String,
    },
}

/// Watching that cannot start: the path it stopped at, and the reason in
/// the operating system's or the `notify` crate's words.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) struct WatchError {
    pub(crate) path: PathBuf,
    pub(crate) reason: String,
}

/// What stops a rebuild, which leaves the last site that built served. Its
/// display is the report, then that the last site is still served.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum RebuildError {
    #[error(fmt = Self::not_rebuilt)]
    Config(ConfigErrors),
    #[error(fmt = Self::not_rebuilt)]
    Input(InputErrors),
}

impl ServeError {
    fn cannot_listen(
        host: &str,
        port: &u16,
        kind: &ListenErrorKind,
        f: &mut Formatter<'_>,
    ) -> FmtResult {
        f.write_str(&match kind {
            ListenErrorKind::PortTaken => {
                fl!(CATALOGUE, "serve-port-taken", port = port.to_string())
            }
            ListenErrorKind::Other { reason } => fl!(
                CATALOGUE,
                "serve-cannot-listen",
                host = host,
                port = port.to_string(),
                reason = reason.as_str()
            ),
        })
    }
}

impl Display for WatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(
            CATALOGUE,
            "serve-cannot-watch",
            path = self.path.to_string_lossy(),
            reason = self.reason.as_str()
        ))
    }
}

impl RebuildError {
    fn not_rebuilt(report: &dyn Display, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{report}")?;
        writeln!(f, "{}", fl!(CATALOGUE, "serve-not-rebuilt"))
    }
}
