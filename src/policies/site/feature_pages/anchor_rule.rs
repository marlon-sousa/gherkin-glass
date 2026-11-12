//! Policy: the anchor rule, the anchors of a feature's page.

use std::collections::HashSet;

use gherkin::{Background, Examples, Rule, Scenario};

use crate::entities::{Anchored, AnchoredFeature, AnchoredRule, AnchoredScenario, ParsedFeature};
use crate::policies::site::feature_pages::example_rows;
use crate::policies::site::{anchor_of, unique};

/// The kinds of block that have an anchor, each named in English for when
/// neither its name nor its keyword leaves anything.
#[derive(Debug, Clone, Copy)]
enum Kind {
    Background,
    Rule,
    Scenario,
    ScenarioOutline,
    Examples,
}

/// The anchors made so far on one page, which a new one must not repeat.
#[derive(Default)]
struct Taken {
    anchors: HashSet<String>,
}

/// `parsed`'s blocks, each with its anchor, and the rows of its outlines' examples with theirs.
pub(crate) fn anchor_feature(parsed: &ParsedFeature) -> AnchoredFeature<'_> {
    let feature = &parsed.feature;
    let mut taken = Taken::default();
    // Blocks take their anchors in file order, which is the order gherkin keeps them in.
    let background = feature
        .background
        .as_ref()
        .map(|background| taken.background(background));
    let mut scenarios = taken.scenarios(&feature.scenarios);
    let mut rules: Vec<_> = feature.rules.iter().map(|rule| taken.rule(rule)).collect();
    // The rows take their anchors after every block.
    for scenario in scenarios
        .iter_mut()
        .chain(rules.iter_mut().flat_map(|rule| &mut rule.scenarios))
    {
        scenario.rows = taken.rows(scenario);
    }
    AnchoredFeature {
        background,
        scenarios,
        rules,
    }
}

impl Taken {
    fn background<'a>(&mut self, background: &'a Background) -> Anchored<'a, Background> {
        Anchored {
            block: background,
            anchor: self.block(&background.name, &background.keyword, Kind::Background),
        }
    }

    fn rule<'a>(&mut self, rule: &'a Rule) -> AnchoredRule<'a> {
        AnchoredRule {
            rule,
            anchor: self.block(&rule.name, &rule.keyword, Kind::Rule),
            background: rule
                .background
                .as_ref()
                .map(|background| self.background(background)),
            scenarios: self.scenarios(&rule.scenarios),
        }
    }

    /// Each with its examples blocks, and no rows yet.
    fn scenarios<'a>(&mut self, scenarios: &'a [Scenario]) -> Vec<AnchoredScenario<'a>> {
        scenarios
            .iter()
            .map(|scenario| self.scenario(scenario))
            .collect()
    }

    fn scenario<'a>(&mut self, scenario: &'a Scenario) -> AnchoredScenario<'a> {
        let kind = if scenario.examples.is_empty() {
            Kind::Scenario
        } else {
            Kind::ScenarioOutline
        };
        AnchoredScenario {
            scenario,
            anchor: self.block(&scenario.name, &scenario.keyword, kind),
            examples: scenario
                .examples
                .iter()
                .map(|examples| self.examples(examples))
                .collect(),
            rows: Vec::new(),
        }
    }

    fn examples<'a>(&mut self, examples: &'a Examples) -> Anchored<'a, Examples> {
        Anchored {
            block: examples,
            anchor: self.block(
                examples.name.as_deref().unwrap_or_default(),
                &examples.keyword,
                Kind::Examples,
            ),
        }
    }

    /// The anchors of the scenarios `outline`'s rows produce, counted from 1.
    fn rows(&mut self, outline: &AnchoredScenario<'_>) -> Vec<String> {
        (1..=example_rows(outline.scenario).len())
            .map(|row| self.unique(format!("{}-row-{row}", outline.anchor)))
            .collect()
    }

    /// The anchor of a block named `name` and written with `keyword`, made unique.
    fn block(&mut self, name: &str, keyword: &str, kind: Kind) -> String {
        let anchor = [anchor_of(name), anchor_of(keyword)]
            .into_iter()
            .find(|anchor| !anchor.is_empty())
            .unwrap_or_else(|| kind.english().to_string());
        self.unique(anchor)
    }

    fn unique(&mut self, anchor: String) -> String {
        unique(anchor, &mut self.anchors)
    }
}

impl Kind {
    fn english(self) -> &'static str {
        match self {
            Kind::Background => "background",
            Kind::Rule => "rule",
            Kind::Scenario => "scenario",
            Kind::ScenarioOutline => "scenario-outline",
            Kind::Examples => "examples",
        }
    }
}

#[cfg(test)]
mod tests;
