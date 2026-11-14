//! Error: what stops ghg serve before it serves, and what stops a rebuild while it serves.

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::PathBuf;

use i18n_embed_fl::fl;
use thiserror::Error;

use super::BuildError;
use crate::policies::CATALOGUE;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum ServeError {
    /// What stops the first build, or, under `--no-build`, the configuration.
    #[error(transparent)]
    Build(#[from] BuildError),
    #[error(fmt = Self::cannot_listen)]
    Listen {
        host: String,
        port: u16,
        kind: ListenErrorKind,
    },
    #[error(transparent)]
    Watch(WatchError),
    #[error(transparent)]
    Signals(SignalsError),
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

/// Listening for the signals that stop ghg serve that cannot start: the reason in the
/// operating system's or the `ctrlc` crate's words.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) struct SignalsError {
    pub(crate) reason: String,
}

/// What stops a rebuild, which leaves the last site that built served. Its
/// display is the report, then that the last site is still served.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) struct RebuildError {
    #[source]
    pub(crate) build: BuildError,
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

impl Display for SignalsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(
            CATALOGUE,
            "serve-cannot-handle-signals",
            reason = self.reason.as_str()
        ))
    }
}

impl Display for RebuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.build)?;
        writeln!(f, "{}", fl!(CATALOGUE, "serve-not-rebuilt"))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    // Rule 68.
    #[case::a_watch_that_cannot_start(
        ServeError::Watch(WatchError {
            path: PathBuf::from("features"),
            reason: "too many folders".into(),
        }),
        "ghg serve cannot watch features for changes: too many folders"
    )]
    // Rule 59.
    #[case::an_address_it_cannot_listen_at(
        ServeError::Listen {
            host: "203.0.113.1".into(),
            port: 8000,
            kind: ListenErrorKind::Other {
                reason: "no such address".into(),
            },
        },
        "ghg serve cannot listen at 203.0.113.1, port 8000: no such address"
    )]
    // Rule 198.
    #[case::signals_it_cannot_handle(
        ServeError::Signals(SignalsError {
            reason: "Operation not permitted".into(),
        }),
        "ghg serve cannot handle Control+C and the signals that stop it: Operation not permitted"
    )]
    fn serve_names_what_it_cannot_watch_listen_at_or_handle(
        #[case] error: ServeError,
        #[case] said: &str,
    ) {
        assert_eq!(error.to_string(), said);
    }
}
