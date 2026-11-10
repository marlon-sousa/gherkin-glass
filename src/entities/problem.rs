//! Entity: the problems that stop a build, each a thing of the input that cannot be read.

use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::{Component, Path};

use thiserror::Error;

/// One thing that cannot be read: a feature file, a `.ghg-ignore` file or a
/// folder of the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// Relative to the input folder, with `/` between folders; the input
    /// folder itself is shown as it was given.
    pub path: String,
    pub line: Option<usize>,
    pub message: String,
}

impl Problem {
    pub(crate) fn new(
        input: &Path,
        path: &Path,
        line: Option<usize>,
        message: impl Into<String>,
    ) -> Self {
        Problem {
            path: display_path(input, path),
            line,
            message: message.into(),
        }
    }
}

/// Folder by folder, so that a folder's contents stay together, then by line.
impl Ord for Problem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path
            .split('/')
            .cmp(other.path.split('/'))
            .then(self.line.cmp(&other.line))
            .then(self.message.cmp(&other.message))
    }
}

impl PartialOrd for Problem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Problem {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.line {
            Some(line) => write!(f, "{}:{}: {}", self.path, line, self.message),
            None => write!(f, "{}: {}", self.path, self.message),
        }
    }
}

/// Every problem of one build, sorted, so that the report is the same on every
/// run. Its display is the report, one line per problem.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub struct Problems(Vec<Problem>);

impl Problems {
    pub(crate) fn new(mut problems: Vec<Problem>) -> Self {
        problems.sort();
        Problems(problems)
    }

    pub fn as_slice(&self) -> &[Problem] {
        &self.0
    }
}

impl Display for Problems {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        for problem in &self.0 {
            writeln!(f, "{problem}")?;
        }
        Ok(())
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

    fn problem(path: &str, line: Option<usize>) -> Problem {
        Problem {
            path: path.to_string(),
            line,
            message: "wrong".to_string(),
        }
    }

    #[test]
    fn problems_are_sorted_whatever_order_they_come_in() {
        // docs/BEHAVIOUR.md, rule 15: the threads that parse may finish in
        // any order.
        let sorted = vec![
            problem("a/x.feature", None),
            problem("a/x.feature", Some(2)),
            problem("a/x.feature", Some(10)),
            problem("a-b.feature", Some(1)),
            problem("b.feature", Some(1)),
        ];
        let mut shuffled = sorted.clone();
        shuffled.reverse();
        shuffled.swap(0, 2);
        assert_eq!(Problems::new(shuffled).as_slice(), sorted.as_slice());
    }

    #[test]
    fn the_input_folder_itself_is_shown_as_given() {
        let input = Path::new("features/");
        assert_eq!(Problem::new(input, input, None, "gone").path, "features/");
        assert_eq!(
            Problem::new(input, &input.join("a").join("b.feature"), None, "x").path,
            "a/b.feature"
        );
    }
}
