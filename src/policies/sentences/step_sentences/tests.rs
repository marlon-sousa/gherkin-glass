mod against_fixtures;

use std::collections::BTreeMap;
use std::path::PathBuf;

use gherkin::{Feature, GherkinEnv};

use super::*;
use crate::entities::ParsedFeature;
use crate::policies::anchor_feature;

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
    let kinds: Vec<_> = find_step_sentences([&anchor_feature(&parsed)])
        .groups
        .iter()
        .map(|group| group.kind)
        .collect();
    assert_eq!(kinds, [StepKind::Given, StepKind::Then]);
}
