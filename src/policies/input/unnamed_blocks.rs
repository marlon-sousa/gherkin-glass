//! Policy: the blocks of a feature that have no name but need one while `allow_unnamed` is `false`.

use gherkin::{Feature, Scenario};

use crate::errors::UnnamedBlock;

/// The line of each unnamed feature, rule, scenario and scenario outline, in file order.
pub(crate) fn unnamed_blocks(feature: &Feature) -> Vec<(usize, UnnamedBlock)> {
    let mut found = Vec::new();
    if feature.name.is_empty() {
        found.push((feature.position.line, UnnamedBlock::Feature));
    }
    add_scenarios(&feature.scenarios, &mut found);
    for rule in &feature.rules {
        if rule.name.is_empty() {
            found.push((rule.position.line, UnnamedBlock::Rule));
        }
        add_scenarios(&rule.scenarios, &mut found);
    }
    found.sort();
    found
}

fn add_scenarios(scenarios: &[Scenario], found: &mut Vec<(usize, UnnamedBlock)>) {
    for scenario in scenarios.iter().filter(|scenario| scenario.name.is_empty()) {
        let block = if scenario.examples.is_empty() {
            UnnamedBlock::Scenario
        } else {
            UnnamedBlock::ScenarioOutline
        };
        found.push((scenario.position.line, block));
    }
}

#[cfg(test)]
mod tests {
    use gherkin::GherkinEnv;

    use super::*;

    #[test]
    fn unnamed_backgrounds_and_examples_are_never_reported() {
        let feature = Feature::parse(
            "Feature:\n\
             \x20 Background:\n\
             \x20   Given a\n\
             \x20 Scenario Outline:\n\
             \x20   Given <n>\n\
             \x20   Examples:\n\
             \x20     | n |\n\
             \x20     | 1 |\n\
             \x20 Rule:\n\
             \x20   Scenario:\n\
             \x20     Given b\n\
             \x20   Scenario: Named\n\
             \x20     Given c\n",
            GherkinEnv::default(),
        )
        .expect("parsed");
        assert_eq!(
            unnamed_blocks(&feature),
            [
                (1, UnnamedBlock::Feature),
                (4, UnnamedBlock::ScenarioOutline),
                (9, UnnamedBlock::Rule),
                (10, UnnamedBlock::Scenario),
            ]
        );
    }
}
