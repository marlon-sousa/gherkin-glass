mod against_fixtures;

use std::collections::BTreeMap;
use std::path::PathBuf;

use gherkin::{Feature, GherkinEnv};

use super::*;
use crate::entities::{PageOptions, ParsedFeature, TagNames};
use crate::policies::site::site_with_feature_pages;
use crate::test_support::revised;

#[test]
fn a_kind_with_no_sentence_has_no_group() {
    let feature = Feature::parse(
        "Feature: F\n  Scenario: S\n    Given a\n    Then b\n",
        GherkinEnv::default(),
    )
    .expect("parsed");
    let parsed = ParsedFeature::new(
        PathBuf::from("f.feature"),
        feature,
        "en".to_string(),
        BTreeMap::new(),
    );
    let site = site_with_feature_pages(
        &[revised(&parsed)],
        None,
        &PageOptions::default(),
        &TagNames::default(),
        None,
    );
    let pages: Vec<_> = site.every_feature_page().collect();
    let kinds: Vec<_> = find_step_sentences(&pages)
        .groups
        .iter()
        .map(|group| group.kind)
        .collect();
    assert_eq!(kinds, [StepKind::Given, StepKind::Then]);
}
