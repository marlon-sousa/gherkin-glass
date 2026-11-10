//! The problems that stop a build: files that cannot be read.

use std::fmt;
use std::path::{Component, Path};

/// One thing that cannot be read: a feature file, a `.ghg-ignore` file or a
/// folder of the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// The path relative to the input folder, with `/` between folders. The
    /// input folder itself is shown as it was given.
    pub path: String,
    /// The line the problem is on, when there is one.
    pub line: Option<usize>,
    /// What is wrong, in the parser's words when the parser found it.
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

/// Problems sort by path, folder by folder, then by line and message.
impl Ord for Problem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path
            .split('/')
            .cmp(other.path.split('/'))
            .then(self.line.cmp(&other.line))
            .then(self.message.cmp(&other.message))
    }
}

impl PartialOrd for Problem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "{}:{}: {}", self.path, line, self.message),
            None => write!(f, "{}: {}", self.path, self.message),
        }
    }
}

/// Every problem of one build, sorted by path and then by line, so that the
/// report is the same on every run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problems(Vec<Problem>);

impl Problems {
    pub(crate) fn new(mut problems: Vec<Problem>) -> Self {
        problems.sort();
        Problems(problems)
    }

    /// The problems, in the order they are reported.
    pub fn as_slice(&self) -> &[Problem] {
        &self.0
    }
}

/// The report: one line per problem.
impl fmt::Display for Problems {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for problem in &self.0 {
            writeln!(f, "{problem}")?;
        }
        Ok(())
    }
}

/// `path` relative to `input`, with `/` between its parts on every system.
pub(crate) fn display_path(input: &Path, path: &Path) -> String {
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
        // any order, so the report sorts what they found.
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
}
