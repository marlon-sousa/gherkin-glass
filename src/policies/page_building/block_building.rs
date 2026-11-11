//! Policy: the blocks of a feature's page, its backgrounds, rules and scenarios, with their steps and tags.

use gherkin::Background;
use i18n_embed_fl::fl;

use super::heading_words::{heading, plain};
use super::step_views::step_views;
use crate::entities::{
    Anchored, AnchoredRule, AnchoredScenario, Block, BlockKind, FromBackground, HeadingLevel,
    HeadingPart, LinkAddress, PageOptions, Paragraph, ParsedFeature, TagNames, TagView,
};
use crate::policies::{CATALOGUE, outside_link};

pub(super) struct Builder<'a> {
    pub(super) parsed: &'a ParsedFeature,
    pub(super) options: &'a PageOptions,
    pub(super) tags: &'a TagNames,
    /// From the page back to the site's root.
    pub(super) root: String,
}

impl Builder<'_> {
    pub(super) fn background(
        &self,
        anchored: &Anchored<'_, Background>,
        level: HeadingLevel,
    ) -> Block {
        let background = anchored.block;
        Block {
            steps: step_views(self.parsed, &background.steps, None),
            description: description(background.description.as_deref()),
            ..block(
                BlockKind::Background,
                level,
                &anchored.anchor,
                heading(&background.keyword, &background.name),
                self.options.collapse_scenarios,
            )
        }
    }

    /// `None` for a background without steps, which adds nothing to a scenario.
    pub(super) fn background_steps(
        &self,
        anchored: &Anchored<'_, Background>,
        introduction: String,
    ) -> Option<FromBackground> {
        let background = anchored.block;
        (!background.steps.is_empty()).then(|| FromBackground {
            id: String::new(),
            introduction,
            anchor: anchored.anchor.clone(),
            heading: heading(&background.keyword, &background.name),
            steps: step_views(self.parsed, &background.steps, None),
            inline: self.options.inline_backgrounds,
        })
    }

    pub(super) fn rule(
        &self,
        anchored: &AnchoredRule<'_>,
        from_feature: &[FromBackground],
    ) -> Block {
        let rule = anchored.rule;
        let mut blocks = Vec::new();
        if let Some(background) = &anchored.background {
            blocks.push(self.background(background, HeadingLevel::Three));
        }
        let mut from: Vec<_> = from_feature.to_vec();
        from.extend(anchored.background.iter().filter_map(|background| {
            self.background_steps(background, fl!(CATALOGUE, "from-rule-background"))
        }));
        blocks.extend(
            anchored
                .scenarios
                .iter()
                .map(|s| self.scenario(s, HeadingLevel::Three, &from)),
        );
        Block {
            tags: self.tags(&rule.tags),
            description: description(rule.description.as_deref()),
            blocks,
            ..block(
                BlockKind::Rule,
                HeadingLevel::Two,
                &anchored.anchor,
                heading(&rule.keyword, &rule.name),
                self.options.collapse_rules,
            )
        }
    }

    pub(super) fn scenario(
        &self,
        anchored: &AnchoredScenario<'_>,
        level: HeadingLevel,
        from: &[FromBackground],
    ) -> Block {
        let scenario = anchored.scenario;
        let anchor = &anchored.anchor;
        let backgrounds = from
            .iter()
            .zip(1..)
            .map(|(from, place)| numbered(from, anchor, place))
            .collect();
        Block {
            tags: self.tags(&scenario.tags),
            description: description(scenario.description.as_deref()),
            backgrounds,
            steps: step_views(self.parsed, &scenario.steps, Some(scenario)),
            blocks: anchored
                .examples
                .iter()
                .map(|examples| self.examples(examples, level.below()))
                .chain(self.examples_as_scenarios(anchored, level.below()))
                .collect(),
            ..block(
                BlockKind::Scenario,
                level,
                anchor,
                heading(&scenario.keyword, &scenario.name),
                self.options.collapse_scenarios,
            )
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

/// The background's steps as the scenario at `anchor` shows them, at `place` among its backgrounds.
fn numbered(from: &FromBackground, anchor: &str, place: usize) -> FromBackground {
    FromBackground {
        id: format!("{anchor}_background_{place}"),
        ..from.clone()
    }
}

pub(super) fn block(
    kind: BlockKind,
    level: HeadingLevel,
    anchor: &str,
    heading: String,
    collapsed: bool,
) -> Block {
    worded_block(kind, level, anchor, vec![plain(&heading)], collapsed)
}

pub(super) fn worded_block(
    kind: BlockKind,
    level: HeadingLevel,
    anchor: &str,
    heading_parts: Vec<HeadingPart>,
    collapsed: bool,
) -> Block {
    Block {
        kind,
        level,
        anchor: anchor.to_string(),
        heading_parts,
        collapsed,
        tags: Vec::new(),
        description: None,
        backgrounds: Vec::new(),
        steps: Vec::new(),
        table: None,
        blocks: Vec::new(),
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
