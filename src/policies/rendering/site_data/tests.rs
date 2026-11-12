//! The search documents, checked against the fixture.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use super::*;
use crate::entities::LinkAddress;
use crate::fixtures::{self, read_features};
use crate::policies::{anchor_feature, search_documents};

fn fixture() -> PathBuf {
    fixtures::fixture("search")
}

fn documents() -> String {
    let features = read_features(&fixture().join("features"));
    let anchored: Vec<_> = features.iter().map(anchor_feature).collect();
    search_documents(&anchored).render()
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

#[test]
fn only_a_feature_s_item_has_a_lang_and_empty_lists_are_left_out_of_the_tree_data() {
    let item = |text: &str, address: &str, lang: Option<&str>, items| TreeItem {
        text: text.into(),
        address: LinkAddress::to(address),
        lang: lang.map(str::to_string),
        items,
    };
    let scenario = TreeItem {
        address: LinkAddress::to("a/b.html").at("s"),
        ..item("Scenario: S", "", None, Vec::new())
    };
    let feature = item("Feature: B", "a/b.html", Some("en"), vec![scenario]);
    let items = [
        item("Site", "index.html", None, Vec::new()),
        item("a", "a/index.html", None, vec![feature]),
    ];
    assert_eq!(
        items.render(),
        "[{\"text\":\"Site\",\"address\":\"index.html\"},\
         {\"text\":\"a\",\"address\":\"a/index.html\",\"items\":[\
         {\"text\":\"Feature: B\",\"address\":\"a/b.html\",\"lang\":\"en\",\"items\":[\
         {\"text\":\"Scenario: S\",\"address\":\"a/b.html#s\"}]}]}]\n"
    );
}
