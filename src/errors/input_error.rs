//! Error: what stops a build in its input, each a thing of the input that cannot be read or is not allowed.

use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::{Component, Path};

use thiserror::Error;

use super::{InputErrorKind, each_on_a_line, located};

/// One thing that cannot be read: a feature file, a `.ghg-ignore` file or a
/// folder of the input; or a block of a feature that needs a name.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) struct InputError {
    /// Relative to the input folder, with `/` between folders; the input
    /// folder itself is shown as it was given.
    pub(crate) path: String,
    pub(crate) line: Option<usize>,
    pub(crate) kind: InputErrorKind,
}

impl InputError {
    /// `path` as an error shows it: relative to `input`, with `/` between folders.
    pub(crate) fn shown_path(input: &Path, path: &Path) -> String {
        display_path(input, path)
    }

    pub(crate) fn new(
        input: &Path,
        path: &Path,
        line: Option<usize>,
        kind: InputErrorKind,
    ) -> Self {
        InputError {
            path: display_path(input, path),
            line,
            kind,
        }
    }
}

impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        located(f, &self.path, self.line, &self.kind.to_string())
    }
}

/// Folder by folder, so that a folder's contents stay together, then by line.
impl Ord for InputError {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path
            .split('/')
            .cmp(other.path.split('/'))
            .then(self.line.cmp(&other.line))
            .then(self.kind.cmp(&other.kind))
    }
}

impl PartialOrd for InputError {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Every error of one build's input, sorted, so that the report is the same
/// on every run. Its display is the report, one line per error.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) struct InputErrors {
    errors: Vec<InputError>,
}

impl InputErrors {
    pub(crate) fn new(mut errors: Vec<InputError>) -> Self {
        errors.sort();
        InputErrors { errors }
    }

    /// Only tests read the errors one by one; the rest of the crate reports them.
    #[cfg(test)]
    pub(crate) fn as_slice(&self) -> &[InputError] {
        &self.errors
    }
}

impl Display for InputErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        each_on_a_line(f, &self.errors)
    }
}

fn display_path(input: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(input).unwrap_or(path);
    let parts: Vec<_> = relative
        .components()
        .filter_map(|part| match part {
            Component::Normal(name) => Some(name.to_string_lossy()),
            _ => None,
        })
        .collect();
    if parts.is_empty() {
        input.display().to_string()
    } else {
        parts.join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error_at(path: &str, line: Option<usize>) -> InputError {
        InputError {
            path: path.to_string(),
            line,
            kind: InputErrorKind::Syntax,
        }
    }

    #[test]
    fn errors_are_sorted_whatever_order_they_come_in() {
        // docs/BEHAVIOUR.md, rule 15: the threads that parse may finish in
        // any order.
        let sorted = vec![
            error_at("a/x.feature", None),
            error_at("a/x.feature", Some(2)),
            error_at("a/x.feature", Some(10)),
            error_at("a-b.feature", Some(1)),
            error_at("b.feature", Some(1)),
        ];
        let mut shuffled = sorted.clone();
        shuffled.reverse();
        shuffled.swap(0, 2);
        assert_eq!(InputErrors::new(shuffled).as_slice(), sorted.as_slice());
    }

    #[test]
    fn the_input_folder_itself_is_shown_as_given() {
        let input = Path::new("features/");
        assert_eq!(
            InputError::new(input, input, None, InputErrorKind::Changed).path,
            "features/"
        );
        assert_eq!(
            InputError::new(
                input,
                &input.join("a").join("b.feature"),
                None,
                InputErrorKind::Changed
            )
            .path,
            "a/b.feature"
        );
    }
}
