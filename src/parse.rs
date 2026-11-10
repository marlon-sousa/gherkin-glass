//! The second step of the build: parse every collected file, in parallel.
//!
//! docs/BEHAVIOUR.md, "What is read", "Doc strings" and "Files that cannot be
//! read".

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use gherkin::{Feature, GherkinEnv, ParseFileError, Step};
use rayon::prelude::*;

use crate::doc_string;
use crate::problem::Problem;

/// A feature file, read.
#[derive(Debug, Clone)]
pub struct ParsedFeature {
    /// The file's path, relative to the input folder.
    pub path: PathBuf,
    /// The feature, as the `gherkin` crate reads it, with every step's doc
    /// string corrected as docs/BEHAVIOUR.md, "Doc strings", says.
    pub feature: Feature,
    /// The media type of each doc string that has one, by the byte offset at
    /// which its step starts.
    media_types: BTreeMap<usize, String>,
}

impl ParsedFeature {
    /// The media type of `step`'s doc string, such as `json`, if it has one.
    pub fn media_type(&self, step: &Step) -> Option<&str> {
        self.media_types.get(&step.span.start).map(String::as_str)
    }
}

/// Parses `files`, found under `input`, each on its own and in parallel.
///
/// Returns the features in the order of `files`, whatever order the threads
/// finish in, and adds a problem for each file that cannot be read.
pub(crate) fn parse(
    input: &Path,
    files: &[PathBuf],
    problems: &mut Vec<Problem>,
) -> Vec<ParsedFeature> {
    let results: Vec<_> = files
        .par_iter()
        .map(|path| parse_file(input, path))
        .collect();
    let mut features = Vec::with_capacity(results.len());
    for result in results {
        match result {
            Ok(feature) => features.push(feature),
            Err(problem) => problems.push(problem),
        }
    }
    features
}

fn parse_file(input: &Path, path: &Path) -> Result<ParsedFeature, Problem> {
    let problem = |line, message: String| Problem::new(input, path, line, message);
    // Each parse gets its own environment: it holds `RefCell`s, so it cannot
    // be shared between threads.
    let mut feature =
        Feature::parse_path(path, GherkinEnv::default()).map_err(|error| match error {
            ParseFileError::Reading { source, .. } => {
                problem(None, format!("cannot be read: {source}"))
            }
            ParseFileError::Parsing { error, source, .. } => {
                let line = parser_line(&source.to_string()).map(|line| within_file(path, line));
                let message = match error {
                    // The crate keeps the line break after an unknown keyword.
                    Some(gherkin::EnvError::UnknownKeyword(keyword)) => {
                        format!("Unknown keyword: '{}'.", keyword.trim_end())
                    }
                    Some(error) => error.to_string(),
                    None => "Syntax error.".to_string(),
                };
                problem(line, message)
            }
        })?;

    // The source is read again only for a feature with doc strings.
    let mut source = None;
    let mut media_types = BTreeMap::new();
    for step in steps_mut(&mut feature) {
        if step.docstring.is_none() {
            continue;
        }
        if source.is_none() {
            let read = fs::read_to_string(path)
                .map_err(|error| problem(None, format!("cannot be read: {error}")))?;
            source = Some(read);
        }
        let source = source.as_deref().expect("read above");
        // The file can only differ from what the crate read if it changed
        // in between.
        let read = doc_string::read(source, step.span.end)
            .ok_or_else(|| problem(None, "changed while it was being read".to_string()))?;
        if let Some(media_type) = read.media_type {
            media_types.insert(step.span.start, media_type);
        }
        step.docstring = Some(read.content);
    }

    Ok(ParsedFeature {
        path: path.strip_prefix(input).unwrap_or(path).to_path_buf(),
        feature,
        media_types,
    })
}

/// The line out of the crate's message, `Error at <line>:<column>: ...`, the
/// only place the crate gives it.
fn parser_line(message: &str) -> Option<usize> {
    let rest = message.strip_prefix("Error at ")?;
    rest.split(':').next()?.parse().ok()
}

/// `line`, or the file's last line if `line` is past it. The crate reports
/// an error at the end of the file on the line after the last one.
fn within_file(path: &Path, line: usize) -> usize {
    match fs::read_to_string(path) {
        Ok(source) => line.min(source.lines().count().max(1)),
        Err(_) => line,
    }
}

/// Every step of `feature`: of its background, its scenarios, and its rules'
/// backgrounds and scenarios.
fn steps_mut(feature: &mut Feature) -> impl Iterator<Item = &mut Step> {
    let Feature {
        background,
        scenarios,
        rules,
        ..
    } = feature;
    let rule_steps = rules.iter_mut().flat_map(|rule| {
        rule.background
            .iter_mut()
            .flat_map(|background| &mut background.steps)
            .chain(
                rule.scenarios
                    .iter_mut()
                    .flat_map(|scenario| &mut scenario.steps),
            )
    });
    background
        .iter_mut()
        .flat_map(|background| &mut background.steps)
        .chain(
            scenarios
                .iter_mut()
                .flat_map(|scenario| &mut scenario.steps),
        )
        .chain(rule_steps)
}
