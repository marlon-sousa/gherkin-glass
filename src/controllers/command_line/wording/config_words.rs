//! Controller: the words of every problem of the configuration, from the interface text catalogue.

use i18n_embed_fl::fl;

use super::{invalid_version_name, line};
use crate::entities::{AccentShortfall, ConfigProblemKind, Theme, ValueType};
use crate::policies::CATALOGUE;
use crate::services::ConfigProblems;

/// One line per problem, the file shown as it was given.
pub(crate) fn config_problems(problems: &ConfigProblems) -> String {
    let path = problems.file.to_string_lossy();
    problems
        .problems
        .iter()
        .map(|problem| line(&path, problem.line, config_message(&problem.kind)))
        .collect()
}

fn config_message(kind: &ConfigProblemKind) -> String {
    match kind {
        ConfigProblemKind::CannotBeRead { reason } => {
            fl!(CATALOGUE, "config-cannot-be-read", reason = reason.as_str())
        }
        ConfigProblemKind::Syntax { message } => message.clone(),
        ConfigProblemKind::UnknownKey { key } => {
            fl!(CATALOGUE, "config-unknown-key", key = key.as_str())
        }
        ConfigProblemKind::Empty { key } => fl!(CATALOGUE, "config-empty", key = key.as_str()),
        ConfigProblemKind::InvalidVersionName { name } => invalid_version_name(name),
        ConfigProblemKind::WrongType {
            key,
            expected: ValueType::Text,
        } => fl!(CATALOGUE, "config-not-text", key = key.as_str()),
        ConfigProblemKind::WrongType {
            key,
            expected: ValueType::Boolean,
        } => fl!(CATALOGUE, "config-not-boolean", key = key.as_str()),
        ConfigProblemKind::NotTableList => fl!(CATALOGUE, "config-not-table-list"),
        ConfigProblemKind::Missing { key } => {
            fl!(CATALOGUE, "config-missing", key = key.as_str())
        }
        ConfigProblemKind::NotRegex { key, reason } => fl!(
            CATALOGUE,
            "config-not-regex",
            key = key.as_str(),
            reason = reason.as_str()
        ),
        ConfigProblemKind::RegexTooLarge { key } => {
            fl!(CATALOGUE, "config-regex-too-large", key = key.as_str())
        }
        ConfigProblemKind::UnknownGroup { key, group } => fl!(
            CATALOGUE,
            "config-unknown-group",
            key = key.as_str(),
            group = group.as_str()
        ),
        ConfigProblemKind::WrongType {
            key,
            expected: ValueType::Colour,
        } => fl!(CATALOGUE, "config-not-colour", key = key.as_str()),
        ConfigProblemKind::TooLittleContrast { key, shortfall } => {
            too_little_contrast(key, shortfall)
        }
    }
}

/// The colours and the ratio are passed as text, so that Fluent never formats them.
fn too_little_contrast(key: &str, shortfall: &AccentShortfall) -> String {
    let colour = shortfall.accent.to_string();
    let background = shortfall.background.to_string();
    let ratio = format!(
        "{}.{:02}",
        shortfall.hundredths / 100,
        shortfall.hundredths % 100
    );
    match shortfall.theme {
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
    }
}
