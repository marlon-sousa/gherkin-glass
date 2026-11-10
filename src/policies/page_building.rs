//! Policy: a feature's page, from the feature as it was read and the settings of every page.

use std::path::{Component, Path};

use gherkin::{Background, Examples, Rule, Scenario, Step, Table};
use i18n_embed_fl::fl;

use crate::entities::{
    Anchors, BackgroundsOption, Block, BlockKind, ContentsEntry, DocStringView, FromBackground,
    Page, PageOptions, PageWords, Paragraph, ParsedFeature, SourceLink, StepView, TableView,
    TagNames, TagView,
};
use crate::policies::{
    CATALOGUE, breadcrumb, make_anchors, outside_link, page_address, page_frame, page_language,
    percent_encoded, root_from, site_title,
};
use crate::policies::{CATALOGUE, highlight_step, make_anchors, page_address, page_language};

/// Stands for the file's path in the words of the source link, to find where
/// the path goes; no message of the catalogue holds it.
const PATH_MARK: &str = "\u{0}";

/// `tags` names every tag of the site.
pub(crate) fn build_page(parsed: &ParsedFeature, options: &PageOptions, tags: &TagNames) -> Page {
    let feature = &parsed.feature;
    let anchors = make_anchors(feature);
    let path = slash_path(&parsed.path);
    let address = page_address(&parsed.path);
    let heading = heading(&feature.keyword, &feature.name);
    let builder = Builder {
        parsed,
        anchors: &anchors,
        options,
        tags,
        root: root_from(&address),
    };
    let mut blocks = Vec::new();
    if let Some(background) = &feature.background {
        blocks.push(builder.background(background, 2));
    }
    let from_feature: Vec<_> = feature
        .background
        .iter()
        .filter_map(|background| {
            builder.background_steps(background, fl!(CATALOGUE, "from-background"))
        })
        .collect();
    blocks.extend(
        feature
            .scenarios
            .iter()
            .map(|s| builder.scenario(s, 2, &from_feature)),
    );
    blocks.extend(
        feature
            .rules
            .iter()
            .map(|rule| builder.rule(rule, &from_feature)),
    );
    let has_background =
        feature.background.is_some() || feature.rules.iter().any(|rule| rule.background.is_some());
    let backgrounds_option = has_background.then(|| BackgroundsOption {
        label: fl!(CATALOGUE, "inline-backgrounds"),
        inline: options.inline_backgrounds,
        enabled: any_background_steps(&blocks),
    });
    let mut parts: Vec<String> = address.split('/').map(str::to_string).collect();
    let file = parts.pop().unwrap_or_default();
    Page {
        frame: page_frame(options),
        breadcrumb: breadcrumb(&site_title(options), &parts, Some((&heading, &file))),
        address,
        language: page_language(&parsed.language).to_string(),
        title: fl!(
            CATALOGUE,
            "page-title",
            heading = heading.as_str(),
            path = path.as_str()
        ),
        heading,
        path: path.clone(),
        tags: builder.tags(&feature.tags),
        description: description(feature.description.as_deref()),
        source: options
            .source_link
            .as_deref()
            .map(|template| source_link(template, &path, feature.position.line)),
        contents: contents(&blocks),
        blocks,
        words: PageWords {
            contents: fl!(CATALOGUE, "page-contents"),
            collapse_all: fl!(CATALOGUE, "collapse-all"),
            expand_all: fl!(CATALOGUE, "expand-all"),
            theme: fl!(CATALOGUE, "theme"),
            theme_system: fl!(CATALOGUE, "theme-system"),
            theme_light: fl!(CATALOGUE, "theme-light"),
            theme_dark: fl!(CATALOGUE, "theme-dark"),
        },
        backgrounds_option,
    }
}

struct Builder<'a> {
    parsed: &'a ParsedFeature,
    anchors: &'a Anchors,
    options: &'a PageOptions,
    tags: &'a TagNames,
    /// From the page back to the site's root.
    root: String,
}

impl Builder<'_> {
    fn background(&self, background: &Background, level: u8) -> Block {
        Block {
            steps: self.steps(&background.steps, None),
            description: description(background.description.as_deref()),
            ..block(
                BlockKind::Background,
                level,
                self.anchors.background(background),
                heading(&background.keyword, &background.name),
                self.options.collapse_scenarios,
            )
        }
    }

    /// `None` for a background without steps, which adds nothing to a scenario.
    fn background_steps(
        &self,
        background: &Background,
        introduction: String,
    ) -> Option<FromBackground> {
        (!background.steps.is_empty()).then(|| FromBackground {
            id: String::new(),
            introduction,
            // `make_anchors` gives every block of the feature an anchor.
            anchor: self
                .anchors
                .background(background)
                .unwrap_or_default()
                .to_string(),
            heading: heading(&background.keyword, &background.name),
            steps: self.steps(&background.steps),
            inline: self.options.inline_backgrounds,
        })
    }

    fn rule(&self, rule: &Rule, from_feature: &[FromBackground]) -> Block {
        let mut blocks = Vec::new();
        if let Some(background) = &rule.background {
            blocks.push(self.background(background, 3));
        }
        let mut from: Vec<_> = from_feature.to_vec();
        from.extend(rule.background.iter().filter_map(|background| {
            self.background_steps(background, fl!(CATALOGUE, "from-rule-background"))
        }));
        blocks.extend(rule.scenarios.iter().map(|s| self.scenario(s, 3, &from)));
        Block {
            tags: self.tags(&rule.tags),
            description: description(rule.description.as_deref()),
            blocks,
            ..block(
                BlockKind::Rule,
                2,
                self.anchors.rule(rule),
                heading(&rule.keyword, &rule.name),
                self.options.collapse_rules,
            )
        }
    }

    fn scenario(&self, scenario: &Scenario, level: u8, from: &[FromBackground]) -> Block {
        // `make_anchors` gives every block of the feature an anchor.
        let anchor = self.anchors.scenario(scenario).unwrap_or_default();
        let backgrounds = from
            .iter()
            .zip(1..)
            .map(|(from, place)| FromBackground {
                id: format!("{anchor}_background_{place}"),
                ..from.clone()
            })
            .collect();
        Block {
            tags: self.tags(&scenario.tags),
            description: description(scenario.description.as_deref()),
            backgrounds,
            steps: self.steps(&scenario.steps),
            steps: self.steps(&scenario.steps, Some(scenario)),
            blocks: scenario
                .examples
                .iter()
                .map(|examples| self.examples(examples, level + 1))
                .collect(),
            ..block(
                BlockKind::Scenario,
                level,
                self.anchors.scenario(scenario),
                heading(&scenario.keyword, &scenario.name),
                self.options.collapse_scenarios,
            )
        }
    }

    fn examples(&self, examples: &Examples, level: u8) -> Block {
        Block {
            tags: self.tags(&examples.tags),
            description: description(examples.description.as_deref()),
            table: examples.table.as_ref().map(table),
            ..block(
                BlockKind::Examples,
                level,
                self.anchors.examples(examples),
                heading(&examples.keyword, examples.name.as_deref().unwrap_or("")),
                false,
            )
        }
    }

    /// Each with its `@`, linked to its page and to the first of `tag_links` that matches it.
    fn tags(&self, tags: &[String]) -> Vec<TagView> {
        tags.iter()
            .map(|tag| {
                let text = format!("@{tag}");
                TagView {
                    address: percent_encoded(&format!("{}{}", self.root, self.tags.address(&text))),
                    outside: outside_link(&text, &self.options.tag_links),
                    text,
                }
            })
            .collect()
    }

    fn steps(&self, steps: &[Step]) -> Vec<StepView> {
    /// `scenario` holds the steps, or is `None` for a background's.
    fn steps(&self, steps: &[Step], scenario: Option<&Scenario>) -> Vec<StepView> {
        steps
            .iter()
            .map(|step| StepView {
                keyword: step.keyword.clone(),
                parts: highlight_step(step, scenario).parts,
                table: step.table.as_ref().map(table),
                doc_string: step.docstring.as_ref().map(|content| DocStringView {
                    media_type: self.parsed.media_type(step).map(str::to_string),
                    content: content.clone(),
                }),
            })
            .collect()
    }
}

fn block(
    kind: BlockKind,
    level: u8,
    anchor: Option<&str>,
    heading: String,
    collapsed: bool,
) -> Block {
    Block {
        kind,
        level,
        // `make_anchors` gives every block of the feature an anchor.
        anchor: anchor.unwrap_or_default().to_string(),
        heading,
        collapsed,
        tags: Vec::new(),
        description: Vec::new(),
        backgrounds: Vec::new(),
        steps: Vec::new(),
        table: None,
        blocks: Vec::new(),
    }
}

/// The keyword as written, then `: ` and the name, or the keyword alone.
fn heading(keyword: &str, name: &str) -> String {
    let keyword = keyword.trim();
    let name = name.trim();
    if name.is_empty() {
        keyword.to_string()
    } else {
        format!("{keyword}: {name}")
    }
}

/// One paragraph of the description's lines, or none: the `gherkin` crate
/// keeps no blank line of a description to tell paragraphs apart.
fn description(text: Option<&str>) -> Vec<Paragraph> {
    let lines: Paragraph = text
        .unwrap_or_default()
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .map(str::to_string)
        .collect();
    if lines.is_empty() {
        Vec::new()
    } else {
        vec![lines]
    }
}

/// The first row is the header; a table has at least one row.
fn table(table: &Table) -> TableView {
    let mut rows = table.rows.iter().cloned();
    TableView {
        header: rows.next().unwrap_or_default(),
        rows: rows.collect(),
    }
}

fn any_background_steps(blocks: &[Block]) -> bool {
    blocks
        .iter()
        .any(|block| !block.backgrounds.is_empty() || any_background_steps(&block.blocks))
}

fn contents(blocks: &[Block]) -> Vec<ContentsEntry> {
    blocks
        .iter()
        .filter(|block| matches!(block.kind, BlockKind::Rule | BlockKind::Scenario))
        .map(|block| ContentsEntry {
            anchor: block.anchor.clone(),
            heading: block.heading.clone(),
            entries: match block.kind {
                BlockKind::Rule => contents(&block.blocks),
                _ => Vec::new(),
            },
        })
        .collect()
}

fn source_link(template: &str, path: &str, line: usize) -> SourceLink {
    let address = template
        .replace("{path}", &percent_encoded(path))
        .replace("{line}", &line.to_string());
    let words = fl!(CATALOGUE, "source-link", path = PATH_MARK);
    let (before, after) = words.split_once(PATH_MARK).unwrap_or((&words, ""));
    SourceLink {
        address,
        before: before.to_string(),
        path: path.to_string(),
        after: after.to_string(),
    }
}

/// The parts of a path relative to the input folder, joined by `/`.
fn slash_path(path: &Path) -> String {
    path.components()
        .filter_map(|part| match part {
            Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use gherkin::{Feature, GherkinEnv};

    use super::*;
    use crate::entities::{OutsideLink, TagLink};
    use crate::policies::{name_tags, tags_of, whole_tag_pattern};

    fn page(text: &str, options: &PageOptions) -> Page {
        let feature = Feature::parse(text, GherkinEnv::default()).expect("parses");
        let parsed = ParsedFeature::new(
            PathBuf::from_iter(["billing", "saque em conta.feature"]),
            feature,
            "en".to_string(),
            BTreeMap::new(),
        );
        let tags = name_tags(&tags_of(std::slice::from_ref(&parsed)));
        build_page(&parsed, options, &tags)
    }

    const FEATURE: &str = "\
@billing
Feature: Refunds
  First line,
  second line.

  Another paragraph.

  Background:
    Given a customer

  Scenario: Outside
    Given x

  Rule: R
    Background: In the rule
      Given y

    Scenario Outline: Inside
      Given <n>

      @in-rule
      Examples:
        | n |
        | 1 |
";

    #[test]
    fn headings_read_the_keyword_and_the_name_at_their_levels() {
        let page = page(FEATURE, &PageOptions::default());
        assert_eq!(page.heading, "Feature: Refunds");
        assert_eq!(
            page.title,
            "Feature: Refunds, billing/saque em conta.feature"
        );
        assert_eq!(page.address, "billing/saque em conta.html");
        let top: Vec<_> = page
            .blocks
            .iter()
            .map(|b| (b.level, b.heading.as_str(), b.anchor.as_str()))
            .collect();
        assert_eq!(
            top,
            [
                (2, "Background", "background"),
                (2, "Scenario: Outside", "outside"),
                (2, "Rule: R", "r"),
            ]
        );
        let rule = &page.blocks[2];
        let inside: Vec<_> = rule
            .blocks
            .iter()
            .map(|b| (b.level, b.anchor.as_str()))
            .collect();
        assert_eq!(inside, [(3, "in-the-rule"), (3, "inside")]);
        let examples = &rule.blocks[1].blocks[0];
        assert_eq!((examples.level, examples.heading.as_str()), (4, "Examples"));
        assert_eq!(
            examples.table,
            Some(TableView {
                header: vec!["n".into()],
                rows: vec![vec!["1".into()]],
            })
        );
    }

    #[test]
    fn the_description_is_a_paragraph_that_keeps_its_lines() {
        let page = page(FEATURE, &PageOptions::default());
        assert_eq!(
            page.tags,
            [TagView {
                text: "@billing".into(),
                address: "../tags/billing.html".into(),
                outside: None,
            }]
        );
        assert_eq!(
            page.description,
            [vec![
                "First line,".to_string(),
                "second line.".to_string(),
                "Another paragraph.".to_string()
            ]]
        );
    }

    #[test]
    fn the_contents_list_rules_and_scenarios_with_a_rule_s_scenarios_inside_it() {
        let page = page(FEATURE, &PageOptions::default());
        let entries: Vec<_> = page
            .contents
            .iter()
            .map(|e| (e.anchor.as_str(), e.entries.len()))
            .collect();
        assert_eq!(entries, [("outside", 0), ("r", 1)]);
        assert_eq!(
            page.contents[1].entries[0].heading,
            "Scenario Outline: Inside"
        );
    }

    #[test]
    fn the_source_link_fills_its_template_with_the_encoded_path_and_the_line() {
        let options = PageOptions {
            source_link: Some("https://x/blob/main/features/{path}#L{line}".into()),
            ..PageOptions::default()
        };
        let source = page(FEATURE, &options).source.expect("a source link");
        assert_eq!(
            source.address,
            "https://x/blob/main/features/billing/saque%20em%20conta.feature#L2"
        );
        assert_eq!(
            (
                source.before.as_str(),
                source.path.as_str(),
                source.after.as_str()
            ),
            ("Source: ", "billing/saque em conta.feature", "")
        );
        assert_eq!(page(FEATURE, &PageOptions::default()).source, None);
    }

    #[test]
    fn the_settings_choose_which_blocks_start_collapsed() {
        let collapsed = |options: &PageOptions| {
            let page = page(FEATURE, options);
            let rule = &page.blocks[2];
            [
                page.blocks[0].collapsed,
                page.blocks[1].collapsed,
                rule.collapsed,
                rule.blocks[0].collapsed,
                rule.blocks[1].collapsed,
                rule.blocks[1].blocks[0].collapsed,
            ]
        };
        assert_eq!(collapsed(&PageOptions::default()), [false; 6]);
        let rules = PageOptions {
            collapse_rules: true,
            ..PageOptions::default()
        };
        assert_eq!(collapsed(&rules), [false, false, true, false, false, false]);
        let scenarios = PageOptions {
            collapse_scenarios: true,
            ..PageOptions::default()
        };
        assert_eq!(
            collapsed(&scenarios),
            [true, true, false, true, true, false]
        );
    }

    #[test]
    fn each_scenario_starts_with_the_feature_s_background_then_its_rule_s() {
        let page = page(FEATURE, &PageOptions::default());
        fn from(block: &Block) -> Vec<(&str, &str, &str, &str, bool)> {
            block
                .backgrounds
                .iter()
                .map(|from| {
                    (
                        from.introduction.as_str(),
                        from.anchor.as_str(),
                        from.heading.as_str(),
                        from.steps[0].text.as_str(),
                        from.inline,
                    )
                })
                .collect()
        }
        let feature = (
            "From the background:",
            "background",
            "Background",
            "a customer",
            true,
        );
        assert!(
            page.blocks[0].backgrounds.is_empty(),
            "the background's own"
        );
        assert_eq!(from(&page.blocks[1]), [feature]);
        let rule = &page.blocks[2];
        assert!(rule.backgrounds.is_empty());
        assert_eq!(
            from(&rule.blocks[1]),
            [
                feature,
                (
                    "From the rule's background:",
                    "in-the-rule",
                    "Background: In the rule",
                    "y",
                    true
                )
            ]
        );
        assert!(rule.blocks[1].blocks[0].backgrounds.is_empty(), "examples");
        assert_eq!(
            page.backgrounds_option,
            Some(BackgroundsOption {
                label: "Show backgrounds inside each scenario".into(),
                inline: true,
                enabled: true,
            })
        );
    }

    #[test]
    fn linked_backgrounds_are_written_so_and_a_background_without_steps_adds_nothing_and_disables_the_option()
     {
        let linked = PageOptions {
            inline_backgrounds: false,
            ..PageOptions::default()
        };
        let linked = page(FEATURE, &linked);
        assert!(!linked.blocks[1].backgrounds[0].inline);
        assert_eq!(linked.backgrounds_option.map(|o| o.inline), Some(false));
        let empty = page(
            "Feature: F\n  Background:\n    Only words.\n\n  Scenario: S\n    Given x\n",
            &PageOptions::default(),
        );
        assert!(empty.blocks[1].backgrounds.is_empty());
        assert_eq!(empty.backgrounds_option.map(|o| o.enabled), Some(false));
        let alone = page(
            "Feature: F\n  Background:\n    Given x\n",
            &PageOptions::default(),
        );
        assert_eq!(alone.backgrounds_option.map(|o| o.enabled), Some(false));
        let none = page(
            "Feature: F\n  Scenario: S\n    Given x\n",
            &PageOptions::default(),
        );
        assert_eq!(none.backgrounds_option, None);
    }

    #[test]
    fn every_tag_links_its_page_and_the_first_outside_link_that_matches() {
        let options = PageOptions {
            tag_links: vec![TagLink {
                source: "@in-(.+)".into(),
                pattern: whole_tag_pattern("@in-(.+)").expect("valid"),
                address: "https://x/$1".into(),
                text: "In $1".into(),
            }],
            ..PageOptions::default()
        };
        let page = page(FEATURE, &options);
        assert_eq!(page.tags[0].outside, None);
        let rule = &page.blocks[2];
        let views: Vec<_> = [&page.tags, &page.blocks[1].tags, &rule.tags]
            .into_iter()
            .flatten()
            .collect();
        assert_eq!(views.len(), 1);
        let examples = &rule.blocks[1].blocks[0];
        let tag = &examples.tags[0];
        assert_eq!(tag.address, "../tags/in-rule.html");
        assert_eq!(
            tag.outside,
            Some(OutsideLink {
                address: "https://x/rule".into(),
                text: "In rule".into(),
            })
        );
    }
}
