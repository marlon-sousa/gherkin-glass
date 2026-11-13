//! Policy: a feature's page, from the feature as it was read and the settings of every page.

mod block_building;
mod example_blocks;
mod heading_words;
mod step_views;

use block_building::{Builder, description, has_scenarios};
use heading_words::heading;
use i18n_embed_fl::fl;

use crate::entities::{
    BackgroundsOption, FeaturePath, HeadingLevel, OwnCrumb, Page, PageContent, PageOptions,
    PageWords, RevisedFeature, SourceLink, TagNames, Trail, percent_encoded,
};
use crate::policies::site::feature_pages::anchor_feature;
use crate::policies::site::page_language;
use crate::policies::{CATALOGUE, page_address};

/// Stands for the file's path in the words of the source link, to find where
/// the path goes; no message of the catalogue holds it.
const PATH_MARK: &str = "\u{0}";

/// The page of `revised` with its blocks, each with its anchor. `tags` names
/// every tag of the site.
pub(crate) fn build_page(
    revised: RevisedFeature<'_>,
    options: &PageOptions,
    tags: &TagNames,
) -> PageContent {
    let RevisedFeature {
        feature: parsed,
        revision,
    } = revised;
    let anchored = anchor_feature(parsed);
    let feature = &parsed.feature;
    let path = FeaturePath::below_input(&parsed.path);
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
            !anchored.scenarios.is_empty() || anchored.rules.iter().any(has_scenarios),
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
        steps_heading: fl!(CATALOGUE, "from-scenario"),
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
        revision,
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

fn source_link(template: &str, path: &FeaturePath, line: usize) -> SourceLink {
    let address = template
        .replace("{path}", &percent_encoded(path.as_str()))
        .replace("{line}", &line.to_string());
    let words = fl!(CATALOGUE, "source-link", path = PATH_MARK);
    let (before, after) = words.split_once(PATH_MARK).unwrap_or((&words, ""));
    SourceLink {
        address,
        before: before.to_string(),
        path: path.clone(),
        after: after.to_string(),
    }
}

#[cfg(test)]
mod tests;
