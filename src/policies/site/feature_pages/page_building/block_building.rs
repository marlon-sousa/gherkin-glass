//! Policy: the blocks of a feature's page, its backgrounds, rules and scenarios, with their steps and tags.

use gherkin::Background;
use i18n_embed_fl::fl;

use super::heading_words::{Kind, heading};
use super::step_views::step_contents;
use crate::entities::{
    Anchored, AnchoredRule, AnchoredScenario, BackgroundBlock, BackgroundContent, BlockHeading,
    HeadingLevel, HeadingPart, LinkAddress, PageOptions, Paragraph, ParsedFeature, RuleBlock,
    RuleContent, ScenarioBlock, ScenarioContent, TagNames, TagView,
};
use crate::policies::CATALOGUE;
use crate::policies::site::outside_link;

pub(super) struct Builder<'a> {
    pub(super) parsed: &'a ParsedFeature,
    pub(super) options: &'a PageOptions,
    pub(super) tags: &'a TagNames,
    /// From the page back to the site's root.
    pub(super) root: String,
}

impl Builder<'_> {
    /// `heading_in_scenario` is the heading of the block that holds its steps
    /// in a scenario that starts with them, and `scenarios_below` whether any
    /// scenario is below the block it opens, the feature or the rule.
    pub(super) fn background(
        &self,
        anchored: &Anchored<'_, Background>,
        level: HeadingLevel,
        heading_in_scenario: String,
        scenarios_below: bool,
    ) -> BackgroundContent {
        let background = anchored.block;
        let collapsed_when_unchecked = (scenarios_below && !background.steps.is_empty())
            .then_some(self.options.collapse_scenarios);
        let collapsed = (collapsed_when_unchecked.is_some() && self.options.inline_backgrounds)
            || self.options.collapse_scenarios;
        BackgroundContent {
            block: BackgroundBlock {
                heading: worded_heading(
                    level,
                    &anchored.anchor,
                    heading(Kind::Background, &background.keyword, &background.name),
                    collapsed,
                ),
                description: description(background.description.as_deref()),
                heading_in_scenario,
                collapsed_when_unchecked,
            },
            steps: step_contents(self.parsed, &background.steps, None),
        }
    }

    pub(super) fn rule(&self, anchored: &AnchoredRule<'_>) -> RuleContent {
        let rule = anchored.rule;
        RuleContent {
            block: RuleBlock {
                heading: worded_heading(
                    HeadingLevel::Two,
                    &anchored.anchor,
                    heading(Kind::Rule, &rule.keyword, &rule.name),
                    self.options.collapse_rules,
                ),
                name: rule.name.clone(),
                tags: self.tags(&rule.tags),
                description: description(rule.description.as_deref()),
            },
            background: anchored.background.as_ref().map(|background| {
                self.background(
                    background,
                    HeadingLevel::Three,
                    fl!(CATALOGUE, "from-rule-background"),
                    has_scenarios(anchored),
                )
            }),
            scenarios: anchored
                .scenarios
                .iter()
                .map(|scenario| self.scenario(scenario, HeadingLevel::Three))
                .collect(),
        }
    }

    pub(super) fn scenario(
        &self,
        anchored: &AnchoredScenario<'_>,
        level: HeadingLevel,
    ) -> ScenarioContent {
        let scenario = anchored.scenario;
        let below = level.below();
        let kind = if scenario.examples.is_empty() {
            Kind::Scenario
        } else {
            Kind::ScenarioOutline
        };
        ScenarioContent {
            block: ScenarioBlock {
                heading: worded_heading(
                    level,
                    &anchored.anchor,
                    heading(kind, &scenario.keyword, &scenario.name),
                    self.options.collapse_scenarios,
                ),
                name: scenario.name.clone(),
                tags: self.tags(&scenario.tags),
                description: description(scenario.description.as_deref()),
            },
            steps: step_contents(self.parsed, &scenario.steps, Some(scenario)),
            examples: anchored
                .examples
                .iter()
                .map(|examples| self.examples(examples))
                .collect(),
            as_scenarios: self.examples_as_scenarios(anchored, below),
        }
    }

    /// Each with its `@`, linked to its page and to the first of `tag_links` that matches it.
    pub(super) fn tags(&self, tags: &[String]) -> Vec<TagView> {
        tags.iter().map(|tag| self.tag(tag)).collect()
    }

    fn tag(&self, tag: &str) -> TagView {
        let text = format!("@{tag}");
        TagView {
            address: LinkAddress::to(&format!("{}{}", self.root, self.tags.address(&text))),
            outside: outside_link(&text, &self.options.tag_links),
            text,
        }
    }
}

pub(super) fn has_scenarios(rule: &AnchoredRule<'_>) -> bool {
    !rule.scenarios.is_empty()
}

pub(super) fn worded_heading(
    level: HeadingLevel,
    anchor: &str,
    parts: Vec<HeadingPart>,
    collapsed: bool,
) -> BlockHeading {
    BlockHeading {
        level,
        anchor: anchor.to_string(),
        parts,
        collapsed,
    }
}

/// The description's lines as one paragraph, or none when it has none: the `gherkin` crate
/// keeps no blank line of a description to tell paragraphs apart.
pub(super) fn description(text: Option<&str>) -> Option<Paragraph> {
    let lines: Paragraph = text
        .unwrap_or_default()
        .lines()
        .map(str::trim_end)
        .filter(has_text)
        .map(str::to_string)
        .collect();
    (!lines.is_empty()).then_some(lines)
}

fn has_text(line: &&str) -> bool {
    !line.trim().is_empty()
}
