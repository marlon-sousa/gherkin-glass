mod against_fixtures;

use std::collections::BTreeMap;
use std::path::PathBuf;

use gherkin::{Feature, GherkinEnv};

use super::*;
use crate::entities::RawFeature;
use crate::errors::ParseError;
use crate::fixtures::MemoryFolders;

/// Parses the text held for a path; a path with no text cannot be opened.
struct FakeParser(BTreeMap<PathBuf, String>);

impl FeatureParser for FakeParser {
    fn parse(&self, path: &Path) -> Result<RawFeature, ParseError> {
        let source = self.0.get(path).cloned().ok_or_else(|| ParseError {
            line: None,
            kind: InputErrorKind::CannotBeRead {
                reason: "permission denied".to_string(),
            },
        })?;
        let feature = Feature::parse(&source, GherkinEnv::default()).map_err(|_| ParseError {
            line: Some(1),
            kind: InputErrorKind::Syntax,
        })?;
        Ok(RawFeature { feature, source })
    }
}

fn read(paths: &[&str], texts: &[(&str, &str)]) -> Result<Vec<ParsedFeature>, InputErrors> {
    read_with(
        MemoryFolders::with_files(paths.iter().copied()),
        texts,
        Unnamed::Allowed,
    )
}

fn read_with(
    folders: MemoryFolders,
    texts: &[(&str, &str)],
    unnamed: Unnamed,
) -> Result<Vec<ParsedFeature>, InputErrors> {
    let parser = FakeParser(
        texts
            .iter()
            .map(|(path, text)| (PathBuf::from(path), text.to_string()))
            .collect(),
    );
    InputService::new(Arc::new(folders), Arc::new(parser)).read(Path::new("in"), unnamed)
}

#[test]
fn the_errors_of_walking_and_parsing_are_reported_together_sorted() {
    // How the service combines the walk's errors with the parser's; the
    // messages are the fakes' own.
    let mut folders = MemoryFolders::with_files([
        "in/a.feature",
        "in/b.feature",
        "in/c.feature",
        "in/locked/d.feature",
    ]);
    folders.unlistable.insert(PathBuf::from("in/locked"));
    let errors = read_with(
        folders,
        &[
            ("in/a.feature", "Feature: A\n"),
            ("in/c.feature", "nothing\n"),
        ],
        Unnamed::Allowed,
    )
    .expect_err("problems");
    assert_eq!(
        errors.to_string(),
        "b.feature: cannot be read: permission denied\n\
         c.feature:1: Syntax error.\n\
         locked: cannot be listed: permission denied\n"
    );
}

#[test]
fn refused_unnamed_blocks_are_reported_with_the_files_that_cannot_be_read() {
    let texts = [
        (
            "in/a.feature",
            "Feature:\n  Scenario: S\n    Given x\n  Scenario:\n    Given y\n",
        ),
        ("in/b.feature", "nothing\n"),
        ("in/c.feature", "Feature: C\n"),
    ];
    let paths = ["in/a.feature", "in/b.feature", "in/c.feature"];
    let errors = read_with(MemoryFolders::with_files(paths), &texts, Unnamed::Refused)
        .expect_err("problems");
    assert_eq!(
        errors.to_string(),
        "a.feature:1: The feature has no name, and allow_unnamed is false.\n\
         a.feature:4: The scenario has no name, and allow_unnamed is false.\n\
         b.feature:1: Syntax error.\n"
    );
    let features = read_with(
        MemoryFolders::with_files(paths.into_iter().take(1)),
        &texts,
        Unnamed::Allowed,
    );
    assert_eq!(features.expect("allowed").len(), 1);
}

#[test]
fn features_keep_the_order_of_their_paths() {
    let texts: Vec<_> = (0..50)
        .map(|n| (format!("in/{n:02}.feature"), format!("Feature: {n}\n")))
        .collect();
    let paths: Vec<_> = texts.iter().map(|(path, _)| path.as_str()).collect();
    let texts: Vec<_> = texts
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();
    let features = read(&paths, &texts).expect("read");
    let names: Vec<_> = features.iter().map(|f| f.feature.name.clone()).collect();
    let expected: Vec<_> = (0..50).map(|n| n.to_string()).collect();
    assert_eq!(names, expected);
}

#[test]
fn a_doc_string_is_corrected_and_its_path_made_relative() {
    let features = read(
        &["in/sub/a.feature"],
        &[(
            "in/sub/a.feature",
            "Feature: A\n  Scenario: S\n    Given text\n      \"\"\"md\n      Hi\n      \"\"\"\n",
        )],
    )
    .expect("read");
    let parsed = &features[0];
    let step = &parsed.feature.scenarios[0].steps[0];
    assert_eq!(parsed.path, Path::new("sub/a.feature"));
    assert_eq!(step.docstring.as_deref(), Some("Hi"));
    assert_eq!(parsed.media_type(step), Some("md"));
}
