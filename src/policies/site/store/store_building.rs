//! Policy: the store of a site's features, each with its object, and the descriptor's tags and step sentences, worked out with the settings of every page.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::entities::{
    BlockContent, DescriptorFeature, DescriptorSentences, DescriptorTag, FeaturePath, ObjectBlock,
    ObjectStep, PageOptions, ParsedFeature, SiteStore, StepKind, TAGS_FOLDER, TagNames,
};
use crate::policies::page_name;
use crate::policies::site::{SentenceKey, name_stored_tags, outside_link, source_address};

use super::feature_object;

/// The store of `features`, sorted by their paths, folder by folder, with the tags of the
/// input in the order of the tag index and the step sentences of each kind.
pub(crate) fn build_store(features: &[&ParsedFeature], options: &PageOptions) -> SiteStore {
    let mut features: Vec<&ParsedFeature> = features.to_vec();
    features.sort_by_cached_key(path_names);
    let names = name_stored_tags(features.iter().copied());
    let tags = names
        .tags()
        .iter()
        .map(|tag| descriptor_tag(tag, &names, options))
        .collect();
    let features: Vec<_> = features
        .into_iter()
        .map(|parsed| descriptor_feature(parsed, options))
        .collect();
    let sentences = sentences_of(&features);
    SiteStore {
        features,
        tags,
        sentences,
    }
}

/// The names of the folders the file is in, then its own, by which paths are sorted.
fn path_names(parsed: &&ParsedFeature) -> Vec<String> {
    FeaturePath::below_input(&parsed.path)
        .names()
        .map(str::to_string)
        .collect()
}

fn descriptor_tag(tag: &str, names: &TagNames, options: &PageOptions) -> DescriptorTag {
    DescriptorTag {
        tag: tag.to_string(),
        page: format!("{TAGS_FOLDER}/{}", names.name(tag)),
        link: outside_link(tag, &options.tag_links),
    }
}

fn descriptor_feature(parsed: &ParsedFeature, options: &PageOptions) -> DescriptorFeature {
    let path = FeaturePath::below_input(&parsed.path);
    let (object, lines) = feature_object(parsed);
    let line = parsed.feature.position.line;
    DescriptorFeature {
        page: page_name(&parsed.path),
        object: Arc::new(object),
        lines,
        row_lines: parsed.row_lines.clone(),
        source: options
            .source_link
            .as_deref()
            .map(|template| source_address(template, &path, line)),
        path,
    }
}

/// The sentences of the steps of every background and scenario of `features`, outlines
/// included, but not of the runs an outline's rows make.
fn sentences_of(features: &[DescriptorFeature]) -> DescriptorSentences {
    let keys: BTreeSet<SentenceKey> = features
        .iter()
        .flat_map(|feature| &feature.object.blocks)
        .flat_map(sentence_keys)
        .collect();
    DescriptorSentences {
        given: of_kind(&keys, StepKind::Given),
        when: of_kind(&keys, StepKind::When),
        then: of_kind(&keys, StepKind::Then),
    }
}

/// The sentences of `keys` of the kind `kind`, in their order.
fn of_kind(keys: &BTreeSet<SentenceKey>, kind: StepKind) -> Vec<String> {
    keys.iter()
        .filter(|key| is_of(key, kind))
        .map(|key| key.text.clone())
        .collect()
}

fn is_of(key: &SentenceKey, kind: StepKind) -> bool {
    key.kind == kind
}

/// The key of each step's sentence in `block` and every block inside it.
fn sentence_keys(block: &ObjectBlock) -> Vec<SentenceKey> {
    match &block.content {
        BlockContent::Background { steps }
        | BlockContent::Scenario { steps }
        | BlockContent::Outline { steps, .. } => steps.iter().map(sentence_key).collect(),
        BlockContent::Rule { blocks } => blocks.iter().flat_map(sentence_keys).collect(),
    }
}

fn sentence_key(step: &ObjectStep) -> SentenceKey {
    SentenceKey::of(step.kind, step.sentence.clone())
}

#[cfg(test)]
mod tests;
