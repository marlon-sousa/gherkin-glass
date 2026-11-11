//! Controller: the words of every problem a run reports, from the interface text catalogue.

mod config_words;
mod input_words;

pub(super) use config_words::config_problems;
use i18n_embed_fl::fl;
pub(super) use input_words::problems;

use crate::entities::{
    ArgumentProblem, BuiltSite, ListenFailure, OutputProblem, OutputProblemKind,
};
use crate::policies::CATALOGUE;
use crate::services::{RebuildFailure, ServeFailure};

/// Empty when the build has nothing to warn of.
pub(super) fn warnings(built: &BuiltSite) -> String {
    built
        .large_search_documents
        .map(|bytes| {
            // Rounded up, so that a size above 5 MB never reads as 5.0.
            let size = format!("{:.1}", (bytes as f64 / 100_000.0).ceil() / 10.0);
            fl!(CATALOGUE, "search-documents-large", size = size.as_str())
        })
        .unwrap_or_default()
}

pub(super) fn argument_problem(problem: &ArgumentProblem) -> String {
    match problem {
        ArgumentProblem::MissingCommand => fl!(CATALOGUE, "missing-command"),
        ArgumentProblem::UnknownCommand { name } => {
            fl!(CATALOGUE, "unknown-command", name = name.as_str())
        }
        ArgumentProblem::UnknownOption { command, option } => fl!(
            CATALOGUE,
            "unknown-option",
            command = command.as_str(),
            option = option.as_str()
        ),
        ArgumentProblem::MissingValue { option } => {
            fl!(CATALOGUE, "missing-value", option = option.as_str())
        }
        ArgumentProblem::UnexpectedValue { option } => {
            fl!(CATALOGUE, "unexpected-value", option = option.as_str())
        }
        ArgumentProblem::RepeatedOption { option } => {
            fl!(CATALOGUE, "repeated-option", option = option.as_str())
        }
        ArgumentProblem::UnexpectedArgument { command, argument } => fl!(
            CATALOGUE,
            "unexpected-argument",
            command = command.as_str(),
            argument = argument.as_str()
        ),
        ArgumentProblem::NotAPort { value } => {
            fl!(CATALOGUE, "not-a-port", value = value.as_str())
        }
        ArgumentProblem::WatchWithoutBuild => fl!(CATALOGUE, "watch-without-build"),
        ArgumentProblem::InvalidVersionName { name } => invalid_version_name(name),
    }
}

/// What stops ghg serve before it serves, as one line, or the report of
/// the configuration or the input.
pub(super) fn serve_failure(failure: &ServeFailure) -> String {
    match failure {
        ServeFailure::Config(problems) => config_problems(problems),
        ServeFailure::Input(found) => problems(found),
        ServeFailure::Output(problem) => output_problem(problem),
        ServeFailure::Listen {
            port,
            failure: ListenFailure::PortTaken,
            ..
        } => fl!(CATALOGUE, "serve-port-taken", port = port.to_string()),
        ServeFailure::Listen {
            host,
            port,
            failure: ListenFailure::Other { reason },
        } => fl!(
            CATALOGUE,
            "serve-cannot-listen",
            host = host.as_str(),
            port = port.to_string(),
            reason = reason.as_str()
        ),
        ServeFailure::Watch(failure) => fl!(
            CATALOGUE,
            "serve-cannot-watch",
            path = failure.path.to_string_lossy(),
            reason = failure.reason.as_str()
        ),
    }
}

/// The report of a rebuild that stopped, then what is still served.
pub(super) fn rebuild_failure(failure: &RebuildFailure) -> String {
    let report = match failure {
        RebuildFailure::Config(problems) => config_problems(problems),
        RebuildFailure::Input(found) => problems(found),
    };
    report + &fl!(CATALOGUE, "serve-not-rebuilt") + "\n"
}

/// The output folder or the version's folder, then what stops the build writing into it.
pub(super) fn output_problem(problem: &OutputProblem) -> String {
    let version = problem.version.as_ref();
    let output = problem.output.to_string_lossy();
    // Joined with `/` on every system.
    let folder = match version {
        Some(name) => format!("{output}/{name}"),
        None => output.into_owned(),
    };
    let message = match &problem.kind {
        OutputProblemKind::AlreadyBuilt => match version {
            Some(name) => fl!(CATALOGUE, "version-already-built", name = name.as_str()),
            None => fl!(CATALOGUE, "site-already-built"),
        },
        OutputProblemKind::HoldsVersions => fl!(CATALOGUE, "site-holds-versions"),
        OutputProblemKind::HoldsSite => fl!(CATALOGUE, "version-holds-site"),
        OutputProblemKind::NotASite => fl!(CATALOGUE, "site-not-a-site"),
        OutputProblemKind::NotAVersion => fl!(CATALOGUE, "version-not-a-version"),
        OutputProblemKind::OtherCapitals => fl!(CATALOGUE, "version-other-capitals"),
        OutputProblemKind::CannotBeRead { reason } => {
            fl!(CATALOGUE, "output-cannot-be-read", reason = reason.as_str())
        }
        OutputProblemKind::CannotBeWritten { reason } => {
            fl!(
                CATALOGUE,
                "output-cannot-be-written",
                reason = reason.as_str()
            )
        }
        OutputProblemKind::CannotBeListed { reason } => {
            fl!(
                CATALOGUE,
                "output-cannot-be-listed",
                reason = reason.as_str()
            )
        }
    };
    line(&folder, None, message)
}

pub(super) fn invalid_version_name(name: &str) -> String {
    fl!(CATALOGUE, "invalid-version-name", name = name)
}

/// The line number is passed as text, so that Fluent never groups its digits.
pub(super) fn line(path: &str, line: Option<usize>, message: String) -> String {
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
    text + "\n"
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::entities::VersionName;

    fn version(version: Option<&str>, kind: OutputProblemKind) -> String {
        output_problem(&OutputProblem {
            output: PathBuf::from("site/out"),
            version: version.map(|name| VersionName::new(name).expect("a version name")),
            kind,
        })
    }

    #[test]
    fn an_output_problem_names_the_output_folder_or_the_version_s() {
        use OutputProblemKind::*;
        let reason = || "denied".to_string();
        assert_eq!(
            version(Some("v1"), AlreadyBuilt),
            "site/out/v1: the version v1 is already built, and replace_existing_version is false.\n"
        );
        assert_eq!(
            version(Some("v1"), NotAVersion),
            "site/out/v1: this is not a version ghg built, so it is left as it is.\n"
        );
        assert_eq!(
            version(Some("v1"), OtherCapitals),
            "site/out/v1: a folder whose name differs from this one only in its capitals is in its place, so it is left as it is.\n"
        );
        assert_eq!(
            version(Some("v1"), CannotBeRead { reason: reason() }),
            "site/out/v1: cannot be read: denied\n"
        );
        assert_eq!(
            version(Some("v1"), CannotBeWritten { reason: reason() }),
            "site/out/v1: cannot be written: denied\n"
        );
        assert_eq!(
            version(None, CannotBeListed { reason: reason() }),
            "site/out: cannot be listed: denied\n"
        );
    }
}
