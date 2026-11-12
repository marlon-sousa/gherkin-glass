//! The search documents as written, checked against the fixture.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use crate::entities::{PageOptions, SiteNode};
use crate::policies::rendering::node_rendering::Render;
use crate::policies::site_tree;
use crate::test_support::{self, read_features};

fn fixture() -> PathBuf {
    test_support::fixture("search")
}

/// The search documents' file, when `node` is it.
fn written_documents(node: &SiteNode) -> Option<String> {
    match node {
        SiteNode::SearchDocuments(documents) => Some(documents.render()),
        _ => None,
    }
}

fn documents() -> String {
    let features = read_features(&fixture().join("features"));
    let tree = site_tree(&features, &PageOptions::default());
    tree.files()
        .find_map(written_documents)
        .expect("a site has its search documents")
}

#[test]
fn every_feature_rule_and_scenario_has_its_document_in_page_and_file_order() {
    // Rules 147 and 148.
    let expected = fs::read_to_string(fixture().join("search.json")).unwrap();
    let expected: Value = serde_json::from_str(&expected).unwrap();
    let written: Value = serde_json::from_str(&documents()).unwrap();
    assert_eq!(written, expected);
}

#[test]
fn the_documents_are_compact_and_keep_letters_outside_ascii() {
    // Rule 149.
    let written = documents();
    assert!(!written.contains('\n'));
    assert!(!written.contains("\": "));
    assert!(!written.contains("\", "));
    assert!(written.contains("poupança"));
    assert!(!written.contains("\\u"));
}

#[test]
fn the_documents_are_the_same_on_every_build() {
    // Rule 149.
    assert_eq!(documents(), documents());
}
