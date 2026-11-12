//! The parts of each step, checked against the fixture.

use std::path::Path;

use gherkin::{Scenario, Step};

use crate::entities::{HighlightedStep, StepPart};
use crate::fixtures::{fixture, read_features};
use crate::policies::highlight_step;

use super::text;

/// Every step of the feature at `path`, in file order, each written with its
/// highlighted parts in braces: `{keyword}`, `{"parameter"}`, `{<placeholder>}`
/// and `{#number}`.
fn steps(path: &str) -> Vec<String> {
    let features = read_features(&fixture("highlighting"));
    let feature = &features
        .iter()
        .find(|feature| feature.path == Path::new(path))
        .expect("the fixture has the feature")
        .feature;
    let mut shown = Vec::new();
    let mut add = |step: &Step, scenario: Option<&Scenario>| {
        let highlighted = highlight_step(step, scenario);
        // Rule 110: the parts are the step's text, unchanged.
        assert_eq!(text(&highlighted), step.value);
        assert_eq!(format!("{} ", highlighted.keyword), step.keyword);
        shown.push(show(&highlighted));
    };
    for step in feature.background.iter().flat_map(|found| &found.steps) {
        add(step, None);
    }
    let rules = feature.rules.iter();
    let backgrounds = rules.clone().filter_map(|rule| rule.background.as_ref());
    let scenarios = feature
        .scenarios
        .iter()
        .chain(rules.flat_map(|rule| &rule.scenarios));
    for step in backgrounds.flat_map(|found| &found.steps) {
        add(step, None);
    }
    for scenario in scenarios {
        for step in &scenario.steps {
            add(step, Some(scenario));
        }
    }
    shown
}

fn show(step: &HighlightedStep) -> String {
    let parts: String = step.parts.iter().map(shown_part).collect();
    format!("{{{}}} {parts}", step.keyword)
}

fn shown_part(part: &StepPart) -> String {
    match part {
        StepPart::Plain(text) => text.clone(),
        StepPart::Number(text) => format!("{{#{text}}}"),
        StepPart::Placeholder(text) => format!("{{{text}}}"),
        StepPart::Parameter(parts) => {
            format!("{{{}}}", parts.iter().map(shown_part).collect::<String>())
        }
    }
}

#[test]
fn every_part_highlighting_finds_and_nothing_else() {
    assert_eq!(
        steps("highlighting.feature"),
        [
            // Rule 112: a background's step has no placeholders.
            r#"{Given} an account with {#100} and a {"<amount>"} note"#,
            // Rule 111.
            r#"{When} the customer types {"hello 42"} and {""}"#,
            r#"{And} the customer types {"a \"quoted\" word"}"#,
            r#"{But} a "lone quote stays plain"#,
            r#"{*} the customer's 'name' is plain"#,
            // Rule 113.
            "{Then} the balance is {#-3.5}, then {#1.000,50} or ${#7} on {#2026}-{#11}-{#10}.",
            "{And} R2D2 is 3rd in v1.2, 2.x, x_1 and item-{#4}",
            r#"{And} the quote in {"a\\" b"} closes after b"#,
            "{Given} <amount> is plain here",
            // Rule 112.
            r#"{Given} {<amount>} in {"{<currency>} 3"} and <unknown>"#,
            "{When} the customer asks for < {#5} >",
            r#"{Given} {#2} {"items"}"#,
        ]
    );
}

#[test]
fn the_keyword_is_highlighted_as_written_in_the_file_s_language() {
    assert_eq!(
        steps("destaque.feature"),
        [
            "{Dado} uma conta com {#100} reais",
            "{Quando} o cliente saca {#20}"
        ]
    );
}
