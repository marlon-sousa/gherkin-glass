//! Policy: a feature's page, from the feature as it was read and the settings of every page.

mod block_building;
mod example_blocks;
mod heading_words;
mod step_views;

use std::path::{Component, Path};

use block_building::{Builder, description};
use heading_words::heading;
use i18n_embed_fl::fl;

use crate::entities::{
    BackgroundContent, BackgroundsOption, HeadingLevel, OwnCrumb, Page, PageContent, PageOptions,
    PageWords, ParsedFeature, RuleContent, ScenarioContent, SourceLink, TagNames, Trail,
    percent_encoded,
};
use crate::policies::site::feature_pages::anchor_feature;
use crate::policies::site::page_language;
use crate::policies::{CATALOGUE, page_address};

/// Stands for the file's path in the words of the source link, to find where
/// the path goes; no message of the catalogue holds it.
const PATH_MARK: &str = "\u{0}";

/// The page of `parsed` with its blocks, each with its anchor. `tags` names
/// every tag of the site.
pub(crate) fn build_page(
    parsed: &ParsedFeature,
    options: &PageOptions,
    tags: &TagNames,
) -> PageContent {
    let anchored = anchor_feature(parsed);
    let feature = &parsed.feature;
    let path = slash_path(&parsed.path);
    let address = page_address(&parsed.path);
    let heading = heading(&feature.keyword, &feature.name);
    let builder = Builder {
        parsed,
        options,
        tags,
        root: address.root(),
    };
    let background = anchored.background.as_ref().map(|background| {
        builder.background(
            background,
            HeadingLevel::Two,
            fl!(CATALOGUE, "from-background"),
        )
    });
    let scenarios: Vec<_> = anchored
        .scenarios
        .iter()
        .map(|scenario| builder.scenario(scenario, HeadingLevel::Two))
        .collect();
    let rules: Vec<_> = anchored
        .rules
        .iter()
        .map(|rule| builder.rule(rule))
        .collect();
    let has_background =
        feature.background.is_some() || feature.rules.iter().any(|rule| rule.background.is_some());
    let backgrounds_option = has_background.then(|| BackgroundsOption {
        label: fl!(CATALOGUE, "inline-backgrounds"),
        inline: options.inline_backgrounds,
        enabled: any_background_steps(background.as_ref(), &scenarios, &rules),
    });
    let page = Page {
        trail: Trail {
            folders: Vec::new(),
            page: Some(OwnCrumb {
                text: heading.clone(),
                file: address.file().to_string(),
            }),
        },
        address,
        language: page_language(&parsed.language).to_string(),
        title: fl!(
            CATALOGUE,
            "page-title",
            heading = heading.as_str(),
            path = path.as_str()
        ),
        heading,
        name: feature.name.clone(),
        path: path.clone(),
        tags: builder.tags(&feature.tags),
        description: description(feature.description.as_deref()),
        source: options
            .source_link
            .as_deref()
            .map(|template| source_link(template, &path, feature.position.line)),
        words: PageWords {
            contents: fl!(CATALOGUE, "page-contents"),
            collapse_all: fl!(CATALOGUE, "collapse-all"),
            expand_all: fl!(CATALOGUE, "expand-all"),
        },
        backgrounds_option,
    };
    PageContent {
        page,
        background,
        scenarios,
        rules,
    }
}

/// Whether a scenario starts with a background's steps: one beside a feature
/// background that has steps, or in a rule whose background has steps.
fn any_background_steps(
    background: Option<&BackgroundContent>,
    scenarios: &[ScenarioContent],
    rules: &[RuleContent],
) -> bool {
    let feature_steps = has_steps(background);
    let in_feature = !scenarios.is_empty() || rules.iter().any(has_scenarios);
    let in_rule = rules.iter().any(starts_with_its_background);
    (feature_steps && in_feature) || in_rule
}

fn has_scenarios(rule: &RuleContent) -> bool {
    !rule.scenarios.is_empty()
}

/// Whether a scenario of `rule` starts with the steps of the rule's background.
fn starts_with_its_background(rule: &RuleContent) -> bool {
    has_steps(rule.background.as_ref()) && has_scenarios(rule)
}

fn has_steps(background: Option<&BackgroundContent>) -> bool {
    background.is_some_and(has_own_steps)
}

fn has_own_steps(background: &BackgroundContent) -> bool {
    !background.steps.is_empty()
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
        .filter_map(normal_name)
        .collect::<Vec<_>>()
        .join("/")
}

fn normal_name(part: Component<'_>) -> Option<String> {
    match part {
        Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
