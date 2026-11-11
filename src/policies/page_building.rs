//! Policy: a feature's page, from the feature as it was read and the settings of every page.

use std::path::{Component, Path};

use gherkin::{Background, Examples, Rule, Scenario, Step, Table};
use i18n_embed_fl::fl;

use crate::entities::{
    Anchors, Block, BlockKind, ContentsEntry, DocStringView, Page, PageOptions, PageWords,
    Paragraph, ParsedFeature, SourceLink, StepView, TableView,
};
use crate::policies::{
    CATALOGUE, breadcrumb, make_anchors, page_address, page_language, percent_encoded, site_title,
};

/// Stands for the file's path in the words of the source link, to find where
/// the path goes; no message of the catalogue holds it.
const PATH_MARK: &str = "\u{0}";

pub(crate) fn build_page(parsed: &ParsedFeature, options: &PageOptions) -> Page {
    let feature = &parsed.feature;
    let anchors = make_anchors(feature);
    let path = slash_path(&parsed.path);
    let heading = heading(&feature.keyword, &feature.name);
    let builder = Builder {
        parsed,
        anchors: &anchors,
        options,
    };
    let mut blocks = Vec::new();
    if let Some(background) = &feature.background {
        blocks.push(builder.background(background, 2));
    }
    blocks.extend(feature.scenarios.iter().map(|s| builder.scenario(s, 2)));
    blocks.extend(feature.rules.iter().map(|rule| builder.rule(rule)));
    let address = page_address(&parsed.path);
    let mut parts: Vec<String> = address.split('/').map(str::to_string).collect();
    let file = parts.pop().unwrap_or_default();
    Page {
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
        tags: tags(&feature.tags),
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
        },
    }
}

struct Builder<'a> {
    parsed: &'a ParsedFeature,
    anchors: &'a Anchors,
    options: &'a PageOptions,
}

impl Builder<'_> {
    fn background(&self, background: &Background, level: u8) -> Block {
        Block {
            steps: self.steps(&background.steps),
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

    fn rule(&self, rule: &Rule) -> Block {
        let mut blocks = Vec::new();
        if let Some(background) = &rule.background {
            blocks.push(self.background(background, 3));
        }
        blocks.extend(rule.scenarios.iter().map(|s| self.scenario(s, 3)));
        Block {
            tags: tags(&rule.tags),
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

    fn scenario(&self, scenario: &Scenario, level: u8) -> Block {
        Block {
            tags: tags(&scenario.tags),
            description: description(scenario.description.as_deref()),
            steps: self.steps(&scenario.steps),
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
            tags: tags(&examples.tags),
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

    fn steps(&self, steps: &[Step]) -> Vec<StepView> {
        steps
            .iter()
            .map(|step| StepView {
                keyword: step.keyword.clone(),
                text: step.value.clone(),
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

fn tags(tags: &[String]) -> Vec<String> {
    tags.iter().map(|tag| format!("@{tag}")).collect()
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

    fn page(text: &str, options: &PageOptions) -> Page {
        let feature = Feature::parse(text, GherkinEnv::default()).expect("parses");
        let parsed = ParsedFeature::new(
            PathBuf::from_iter(["billing", "saque em conta.feature"]),
            feature,
            "en".to_string(),
            BTreeMap::new(),
        );
        build_page(&parsed, options)
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
        assert_eq!(page.tags, ["@billing"]);
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
}
