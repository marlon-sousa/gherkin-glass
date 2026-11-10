//! Entity: the problems that stop a build, each a thing of the input that cannot be read or is not allowed.

use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::{Component, Path};

use thiserror::Error;

/// One thing that cannot be read: a feature file, a `.ghg-ignore` file or a
/// folder of the input; or a block of a feature that needs a name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// Relative to the input folder, with `/` between folders; the input
    /// folder itself is shown as it was given.
    pub path: String,
    pub line: Option<usize>,
    pub kind: ProblemKind,
}

/// What is wrong. The words a person reads come from the interface text
/// catalogue; the `#[error]` text is for developers. Every `String` here is
/// another program's words, passed through as they are.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum ProblemKind {
    /// The input folder itself does not exist.
    #[error("the input folder does not exist")]
    InputMissing,
    #[error("cannot be listed: {reason}")]
    CannotBeListed { reason: String },
    #[error("cannot be read: {reason}")]
    CannotBeRead { reason: String },
    /// `reason` is the `ignore` crate's.
    #[error("{reason}")]
    InvalidRule { reason: String },
    /// `message` is the `gherkin` crate's.
    #[error("{message}")]
    Parser { message: String },
    /// The parser named only what it expected, in an order that changes from run to run.
    #[error("Syntax error.")]
    Syntax,
    /// The file changed between its parse and the read of its source.
    #[error("changed while it was being read")]
    Changed,
    /// Only while `allow_unnamed` is `false`.
    #[error("unnamed {block:?}")]
    Unnamed { block: UnnamedBlock },
    /// `page` is the address both files would have, and `first` the path,
    /// shown as a problem's is, of the file that has it.
    #[error("has the same page, {page}, as {first}")]
    SamePage { page: String, first: String },
    /// A file of the site; `reason` is the operating system's.
    #[error("cannot be written: {reason}")]
    CannotBeWritten { reason: String },
}

/// The blocks that need a name while `allow_unnamed` is `false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnnamedBlock {
    Feature,
    Rule,
    Scenario,
    /// A scenario with examples.
    ScenarioOutline,
}

impl Problem {
    /// A problem with a file of the site, whose path is the output folder as
    /// it was given, followed by the file's path in it.
    pub(crate) fn of_site(output: &Path, file: &str, kind: ProblemKind) -> Self {
        let output = output.display().to_string();
        let path = if output.is_empty() || output.ends_with('/') {
            format!("{output}{file}")
        } else {
            format!("{output}/{file}")
        };
        Problem {
            path,
            line: None,
            kind,
        }
    }

    pub(crate) fn new(input: &Path, path: &Path, line: Option<usize>, kind: ProblemKind) -> Self {
        Problem {
            path: display_path(input, path),
            line,
            kind,
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
            .then(self.kind.cmp(&other.kind))
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
            Some(line) => write!(f, "{}:{}: {}", self.path, line, self.kind),
            None => write!(f, "{}: {}", self.path, self.kind),
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
            kind: ProblemKind::Syntax,
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
        assert_eq!(
            Problem::new(input, input, None, ProblemKind::Changed).path,
            "features/"
        );
        assert_eq!(
            Problem::new(
                input,
                &input.join("a").join("b.feature"),
                None,
                ProblemKind::Changed
            )
            .path,
            "a/b.feature"
        );
    }
}
