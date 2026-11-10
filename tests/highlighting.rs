//! docs/BEHAVIOUR.md, "Highlighting and themes", "Highlighting", checked against the fixture.

use std::path::Path;
use std::sync::Arc;

use gherkin::{Scenario, Step};
use gherkin_glass::{
    GherkinParser, HighlightedStep, InputService, InputWalker, StepPart, Unnamed, highlight_step,
};

/// Every step of the feature at `path`, in file order, each written with its
/// highlighted parts in braces: `{keyword}`, `{"parameter"}`, `{<placeholder>}`
/// and `{#number}`.
fn steps(path: &str) -> Vec<String> {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/highlighting");
    let features = InputService::new(Arc::new(InputWalker), Arc::new(GherkinParser))
        .read(&input, Unnamed::Allowed)
        .unwrap_or_else(|problems| panic!("the input is read:\n{problems}"));
    let feature = &features
        .iter()
        .find(|feature| feature.path == Path::new(path))
        .expect("the fixture has the feature")
        .feature;
    let mut shown = Vec::new();
    let mut add = |step: &Step, scenario: Option<&Scenario>| {
        let highlighted = highlight_step(step, scenario);
        // Rule 37: the parts are the step's text, unchanged.
        assert_eq!(highlighted.text(), step.value);
        assert_eq!(format!("{} ", highlighted.keyword), step.keyword);
        shown.push(show(&highlighted));
    };
    for step in feature.background.iter().flat_map(|found| &found.steps) {
        add(step, None);
    }
    for scenario in &feature.scenarios {
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
            // Rule 39: a background's step has no placeholders.
            r#"{Given} an account with {#100} and a {"<amount>"} note"#,
            // Rule 38.
            r#"{When} the customer types {"hello 42"} and {""}"#,
            r#"{And} the customer types {"a \"quoted\" word"}"#,
            r#"{But} a "lone quote stays plain"#,
            r#"{*} the customer's 'name' is plain"#,
            // Rule 40.
            "{Then} the balance is {#-3.5}, then {#1.000,50} or ${#7} on {#2026}-{#11}-{#10}.",
            "{And} R2D2 is 3rd in v1.2, x_1 and item-{#4}",
            "{Given} <amount> is plain here",
            // Rule 39.
            r#"{Given} {<amount>} in {"{<currency>} 3"} and <unknown>"#,
            "{When} the customer asks for < {#5} >",
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
