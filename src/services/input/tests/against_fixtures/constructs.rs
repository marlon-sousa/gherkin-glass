//! What the input service reads of a feature, checked against the fixtures.

use super::*;

// What is read: rules 6 to 8.

#[test]
fn every_construct_is_read() {
    // Rule 6.
    let features = read(&fixture("constructs"));
    let feature = &features
        .iter()
        .find(|feature| feature.path == Path::new("every-construct.feature"))
        .expect("the feature is read")
        .feature;

    assert_eq!(feature.name, "Every construct");
    assert_eq!(feature.tags, ["billing", "slow"]);
    assert_eq!(
        feature.description.as_deref(),
        Some("A description of the feature,\nover two lines.")
    );
    let background = feature.background.as_ref().expect("a background");
    assert_eq!(background.name, "A logged in customer");

    let plain = &feature.scenarios[0];
    assert_eq!(plain.name, "A plain scenario");
    assert_eq!(
        plain.description.as_deref(),
        Some("A description of the scenario.")
    );
    let keywords: Vec<_> = plain.steps.iter().map(|step| step.keyword.trim()).collect();
    assert_eq!(keywords, ["Given", "When", "Then", "And", "But", "*"]);
    let table = plain.steps[0].table.as_ref().expect("a data table");
    assert_eq!(
        table.rows,
        [["item", "price"], ["pen", "1.50"], ["a | b", "2"]]
    );

    let outline = &feature.scenarios[1];
    assert_eq!(outline.keyword, "Scenario Outline");
    assert_eq!(outline.tags, ["outline"]);
    assert_eq!(outline.examples.len(), 2);
    assert_eq!(outline.examples[0].name.as_deref(), Some("Cheap"));
    assert_eq!(outline.examples[0].tags, ["cheap"]);
    assert_eq!(
        outline.examples[1].table.as_ref().expect("a table").rows,
        [["count", "total"], ["9", "90"]]
    );

    let rule = &feature.rules[0];
    assert_eq!(rule.name, "A rule with its own background");
    assert_eq!(rule.description.as_deref(), Some("The rule's description."));
    assert_eq!(
        rule.background
            .as_ref()
            .expect("the rule's background")
            .name,
        "Inside the rule"
    );
    assert_eq!(rule.scenarios[0].tags, ["in-rule"]);
    assert_eq!(
        rule.scenarios[0].steps[0].docstring.as_deref(),
        Some("Some text.")
    );
}

#[test]
fn a_feature_is_read_in_its_own_language() {
    // Rule 7.
    let features = read(&fixture("constructs"));
    let feature = &features
        .iter()
        .find(|feature| feature.path == Path::new("em-portugues.feature"))
        .expect("the feature is read")
        .feature;

    assert_eq!(feature.keyword, "Funcionalidade");
    assert_eq!(feature.name, "Uma funcionalidade em português");
    assert_eq!(
        feature.background.as_ref().expect("a background").keyword,
        "Contexto"
    );
    let keywords: Vec<_> = feature.scenarios[0]
        .steps
        .iter()
        .map(|step| step.keyword.trim())
        .collect();
    assert_eq!(keywords, ["Quando", "Então", "E"]);
}

#[test]
fn comments_are_not_kept() {
    // Rule 8: the fixture has comments at the top, inside the feature's
    // description and between two steps.
    let features = read(&fixture("constructs"));
    let feature = &features
        .iter()
        .find(|feature| feature.path == Path::new("every-construct.feature"))
        .expect("the feature is read")
        .feature;
    assert_eq!(
        feature.description.as_deref(),
        Some("A description of the feature,\nover two lines.")
    );
    let steps: Vec<_> = feature.scenarios[0]
        .steps
        .iter()
        .map(|step| step.value.as_str())
        .collect();
    assert_eq!(
        steps,
        [
            "an order with these lines:",
            "the order is placed",
            "a receipt is sent",
            "no error is shown",
            "nothing is charged twice",
            "the order is saved",
        ]
    );
}
