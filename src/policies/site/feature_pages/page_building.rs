//! Policy: a feature's page, from the feature as it was read and the settings of every page.

mod block_building;
mod example_blocks;
mod heading_words;
mod step_views;

use std::path::{Component, Path};

use block_building::{Builder, description};
pub(crate) use heading_words::heading;
use i18n_embed_fl::fl;

use crate::entities::{
    AnchoredFeature, BackgroundsOption, BlockHeading, ContentsEntry, FeatureBlock, HeadingLevel,
    Page, PageOptions, PageWords, ScenarioBlock, SourceLink, TagNames, percent_encoded,
};
use crate::policies::site::{breadcrumb, page_language, site_title};
use crate::policies::{CATALOGUE, page_address};

/// Stands for the file's path in the words of the source link, to find where
/// the path goes; no message of the catalogue holds it.
const PATH_MARK: &str = "\u{0}";

/// `tags` names every tag of the site, and `folders` are the names of the
/// folder nodes the page is in, from the outermost, for its breadcrumb.
pub(crate) fn build_page(
    anchored: &AnchoredFeature<'_>,
    options: &PageOptions,
    tags: &TagNames,
    folders: &[String],
) -> Page {
    let parsed = anchored.parsed;
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
    let mut blocks = Vec::new();
    if let Some(background) = &anchored.background {
        blocks.push(FeatureBlock::Background(
            builder.background(background, HeadingLevel::Two),
        ));
    }
    let from_feature: Vec<_> = anchored
        .background
        .iter()
        .filter_map(|background| {
            builder.background_steps(background, fl!(CATALOGUE, "from-background"))
        })
        .collect();
    blocks.extend(
        anchored
            .scenarios
            .iter()
            .map(|s| builder.scenario(s, HeadingLevel::Two, &from_feature))
            .map(FeatureBlock::Scenario),
    );
    blocks.extend(
        anchored
            .rules
            .iter()
            .map(|rule| builder.rule(rule, &from_feature))
            .map(FeatureBlock::Rule),
    );
    let has_background =
        feature.background.is_some() || feature.rules.iter().any(|rule| rule.background.is_some());
    let backgrounds_option = has_background.then(|| BackgroundsOption {
        label: fl!(CATALOGUE, "inline-backgrounds"),
        inline: options.inline_backgrounds,
        enabled: any_background_steps(&blocks),
    });
    Page {
        breadcrumb: breadcrumb(
            &site_title(options),
            folders,
            Some((&heading, address.file())),
        ),
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
        },
        backgrounds_option,
    }
}

fn any_background_steps(blocks: &[FeatureBlock]) -> bool {
    blocks.iter().any(has_background_steps)
}

/// Whether `block` is or holds a scenario that starts with a background's steps.
fn has_background_steps(block: &FeatureBlock) -> bool {
    match block {
        FeatureBlock::Background(_) => false,
        FeatureBlock::Rule(rule) => rule.scenarios.iter().any(starts_with_background),
        FeatureBlock::Scenario(scenario) => starts_with_background(scenario),
    }
}

fn starts_with_background(scenario: &ScenarioBlock) -> bool {
    !scenario.backgrounds.is_empty()
}

/// Rules and scenarios are listed in a page's contents, and a rule's entry lists its scenarios.
fn contents(blocks: &[FeatureBlock]) -> Vec<ContentsEntry> {
    blocks.iter().filter_map(contents_entry).collect()
}

fn contents_entry(block: &FeatureBlock) -> Option<ContentsEntry> {
    match block {
        FeatureBlock::Background(_) => None,
        FeatureBlock::Rule(rule) => Some(ContentsEntry {
            entries: rule.scenarios.iter().map(scenario_entry).collect(),
            ..entry(&rule.heading)
        }),
        FeatureBlock::Scenario(scenario) => Some(scenario_entry(scenario)),
    }
}

fn scenario_entry(scenario: &ScenarioBlock) -> ContentsEntry {
    entry(&scenario.heading)
}

fn entry(heading: &BlockHeading) -> ContentsEntry {
    ContentsEntry {
        anchor: heading.anchor.clone(),
        heading: heading.text(),
        entries: Vec::new(),
    }
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
