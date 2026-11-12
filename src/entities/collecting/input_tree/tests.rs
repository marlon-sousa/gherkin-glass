use std::collections::BTreeMap;
use std::path::Path;

use gherkin::{Feature, GherkinEnv};

use super::*;
use crate::errors::{InputError, InputErrorKind};

fn error(path: &str, kind: InputErrorKind) -> InputError {
    InputError::new(Path::new("in"), &Path::new("in").join(path), None, kind)
}

fn feature(path: &str) -> ParsedFeature {
    let feature =
        Feature::parse(format!("Feature: {path}\n"), GherkinEnv::default()).expect("a feature");
    ParsedFeature::new(path.into(), feature, "en".into(), BTreeMap::new())
}

/// A tree of `in` with `a.feature`, `b.feature` and `sub/c.feature` allowed, `notes.md` not
/// allowed, and `locked`, a folder that could not be listed.
fn tree() -> InputTree {
    let mut builder = InputTreeBuilder::listed("in".into(), Vec::new());
    let root = builder.root();
    builder.add_allowed_file(root, "in/a.feature".into());
    builder.add_allowed_file(root, "in/b.feature".into());
    builder.add_file_not_allowed(root, "in/notes.md".into());
    let sub = builder.add_listed_folder(root, "in/sub".into(), Vec::new());
    builder.add_allowed_file(sub, "in/sub/c.feature".into());
    let unreadable = error(
        "locked",
        InputErrorKind::CannotBeListed {
            reason: "denied".into(),
        },
    );
    builder.add_unlisted_folder(root, "in/locked".into(), unreadable);
    builder.build()
}

fn paths(files: &[FileToParse]) -> Vec<&Path> {
    files.iter().map(|file| file.path.as_path()).collect()
}

/// `file` parsed into a feature named after its path.
fn parsed(file: FileToParse) -> ParsedFile {
    let result = Ok(feature(&file.path.display().to_string()));
    ParsedFile { file, result }
}

/// `file` parsed into a feature, or a syntax error when it is `b.feature`.
fn parsed_with_b_broken(file: FileToParse) -> ParsedFile {
    if file.path.ends_with("b.feature") {
        let result = Err(vec![error("b.feature", InputErrorKind::Syntax)]);
        ParsedFile { file, result }
    } else {
        parsed(file)
    }
}

fn feature_path(feature: &ParsedFeature) -> String {
    feature.path.display().to_string()
}

#[test]
fn only_allowed_files_with_nothing_parsed_are_handed_out_in_file_order() {
    let mut tree = tree();
    let files = tree.files_to_parse();
    assert_eq!(
        paths(&files),
        [
            Path::new("in/a.feature"),
            Path::new("in/b.feature"),
            Path::new("in/sub/c.feature")
        ]
    );
    tree.put_parsed(files.into_iter().take(1).map(parsed).collect());
    assert_eq!(
        paths(&tree.files_to_parse()),
        [Path::new("in/b.feature"), Path::new("in/sub/c.feature")]
    );
    assert_eq!(tree.allowed_files().len(), 3);
}

#[test]
fn every_error_the_tree_holds_is_reported_sorted_and_only_features_parsed_are_given() {
    // Rule 15: the folders' errors, the pages' and parsing's, together.
    let mut tree = tree();
    let files = tree.files_to_parse();
    tree.put_parsed(files.into_iter().map(parsed_with_b_broken).collect());
    let clash = InputErrorKind::SamePage {
        page: "sub/c.html".into(),
        first: "a.feature".into(),
    };
    tree.put_page_errors(vec![PageClash {
        path: "in/sub/c.feature".into(),
        error: error("sub/c.feature", clash),
    }]);
    let errors = tree.input_errors().expect("errors");
    let reported: Vec<_> = errors.as_slice().iter().map(shown).collect();
    assert_eq!(reported, ["b.feature", "locked", "sub/c.feature"]);
    let features: Vec<_> = tree.features().map(feature_path).collect();
    assert_eq!(features, ["in/a.feature", "in/sub/c.feature"]);
}

#[test]
fn a_tree_whose_files_are_all_parsed_holds_no_error() {
    let mut builder = InputTreeBuilder::listed("in".into(), Vec::new());
    let root = builder.root();
    builder.add_allowed_file(root, "in/a.feature".into());
    let mut tree = builder.build();
    let files = tree.files_to_parse();
    tree.put_parsed(files.into_iter().map(parsed).collect());
    assert!(tree.input_errors().is_none());
    assert!(tree.files_to_parse().is_empty());
}

#[test]
fn an_input_folder_that_could_not_be_listed_holds_only_its_error() {
    let missing = error("", InputErrorKind::InputMissing);
    let tree = InputTree::unlisted_input("in".into(), missing.clone());
    assert!(tree.allowed_files().is_empty());
    assert_eq!(tree.input_errors(), Some(InputErrors::new(vec![missing])));
}

#[test]
fn page_errors_replace_those_the_nodes_held_and_one_with_no_node_is_kept_at_the_root() {
    let mut tree = tree();
    let clash = |path: &str| PageClash {
        path: Path::new("in").join(path),
        error: error(path, InputErrorKind::InTagsFolder { page: path.into() }),
    };
    tree.put_page_errors(vec![clash("a.feature"), clash("sub")]);
    tree.put_page_errors(vec![clash("sub"), clash("gone.feature")]);
    let root_errors: Vec<_> = tree
        .tree
        .root()
        .value()
        .page_errors
        .iter()
        .map(shown)
        .collect();
    assert_eq!(root_errors, ["gone.feature"]);
    let errors = tree.input_errors().expect("errors");
    let reported: Vec<_> = errors.as_slice().iter().map(shown).collect();
    assert_eq!(reported, ["gone.feature", "locked", "sub"]);
}

fn shown(error: &InputError) -> &str {
    &error.path
}

#[test]
fn a_file_handed_out_by_another_tree_changes_nothing() {
    let mut builder = InputTreeBuilder::listed("other".into(), Vec::new());
    let root = builder.root();
    builder.add_allowed_file(root, "other/x.feature".into());
    let other = builder.build();
    let mut tree = tree();
    tree.put_parsed(other.files_to_parse().into_iter().map(parsed).collect());
    assert_eq!(tree.files_to_parse().len(), 3);
    assert!(tree.features().next().is_none());
}
