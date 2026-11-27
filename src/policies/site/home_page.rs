//! Policy: the home page a shell of a site without versions or of a preview holds, from the site's store: the totals, the links to the tags' index and the step sentences, and the folders and features at the top of the input, each feature with its contents, every link an address of the shell's own pages.

use std::collections::BTreeSet;

use i18n_embed_fl::fl;

use super::block_counts::{is_rule, is_scenario, rule_count, scenario_count};
use crate::entities::{
    BlockContent, BlockKind, ContentsEntry, DescriptorFeature, FeatureEntry, FeatureObject,
    FolderLink, FolderWords, HomePage, LinkAddress, ObjectBlock, PageOptions, SiteScope, SiteStore,
    percent_encoded, plain_parts, text_of,
};
use crate::policies::site::heading;
use crate::policies::{CATALOGUE, site_title};

/// The `id` of the group of the scenarios outside every rule, which no anchor takes.
const OUTSIDE_RULES: &str = "outside_rules";

/// The home page of the site of `store`, built with the settings of every page `options`.
pub(crate) fn build_home_page(store: &SiteStore, options: &PageOptions) -> HomePage {
    let folders: BTreeSet<&str> = store
        .features
        .iter()
        .flat_map(|feature| folders_of(feature.path.as_str()))
        .collect();
    let top: BTreeSet<&str> = folders
        .iter()
        .filter(|folder| !folder.contains('/'))
        .copied()
        .collect();
    let mut features: Vec<&DescriptorFeature> = store
        .features
        .iter()
        .filter(|feature| !feature.path.as_str().contains('/'))
        .collect();
    features.sort_by(|one, other| one.path.as_str().cmp(other.path.as_str()));
    let objects = || store.features.iter().map(|feature| feature.object.as_ref());
    HomePage {
        title: site_title(options),
        totals: vec![
            fl!(CATALOGUE, "folder-count", count = folders.len()),
            fl!(CATALOGUE, "feature-count", count = store.features.len()),
            fl!(
                CATALOGUE,
                "rule-count",
                count = objects().map(rule_count).sum::<usize>()
            ),
            fl!(
                CATALOGUE,
                "scenario-count",
                count = objects().map(scenario_count).sum::<usize>()
            ),
        ],
        tags: FolderLink {
            address: LinkAddress::of_page("tags/"),
            name: fl!(CATALOGUE, "tag-index-title"),
        },
        sentences: FolderLink {
            address: LinkAddress::of_page("steps"),
            name: fl!(CATALOGUE, "sentences-title"),
        },
        nothing_given: (options.scope == SiteScope::OnlyListed && store.features.is_empty())
            .then(|| fl!(CATALOGUE, "only-nothing-given")),
        words: FolderWords {
            folders: fl!(CATALOGUE, "folders"),
            features: fl!(CATALOGUE, "features"),
        },
        folders: top.into_iter().map(folder_link).collect(),
        features: features.into_iter().map(feature_entry).collect(),
    }
}

/// The folders that hold the file at `path`, from the outermost: `a` and `a/b` for `a/b/c`.
fn folders_of(path: &str) -> impl Iterator<Item = &str> {
    path.match_indices('/')
        .filter_map(|(end, _)| path.get(..end))
}

fn folder_link(name: &str) -> FolderLink {
    FolderLink {
        address: LinkAddress::of_page(&format!("{name}/")),
        name: name.to_string(),
    }
}

fn feature_entry(feature: &DescriptorFeature) -> FeatureEntry {
    let object = &feature.object;
    let count = scenario_count(object);
    let heading = heading(BlockKind::Feature, &object.text.name);
    let address = LinkAddress::of_page(&feature.page);
    FeatureEntry {
        label: fl!(
            CATALOGUE,
            "feature-scenarios",
            count = count,
            feature = text_of(&heading)
        ),
        scenarios: fl!(CATALOGUE, "scenario-count", count = count),
        contents: contents_of(object),
        language: object.lang.clone(),
        heading,
        address,
    }
}

/// The contents of `object`'s page: its scenarios, in the group of those outside every rule
/// when it has a rule, then its rules, each with its scenarios.
fn contents_of(object: &FeatureObject) -> Vec<ContentsEntry> {
    let scenarios: Vec<ContentsEntry> = scenarios_of(&object.blocks).collect();
    let rules = object.blocks.iter().filter_map(rule_entry);
    if scenarios.is_empty() || !object.blocks.iter().any(is_rule) {
        return scenarios.into_iter().chain(rules).collect();
    }
    let outside = ContentsEntry {
        anchor: OUTSIDE_RULES.to_string(),
        heading: plain_parts(&fl!(CATALOGUE, "outside-rules")),
        entries: scenarios,
    };
    [outside].into_iter().chain(rules).collect()
}

fn scenarios_of(blocks: &[ObjectBlock]) -> impl Iterator<Item = ContentsEntry> + '_ {
    blocks
        .iter()
        .filter(|block| is_scenario(block))
        .map(|block| entry(block, Vec::new()))
}

fn rule_entry(block: &ObjectBlock) -> Option<ContentsEntry> {
    match &block.content {
        BlockContent::Rule { blocks } => Some(entry(block, scenarios_of(blocks).collect())),
        _ => None,
    }
}

fn entry(block: &ObjectBlock, entries: Vec<ContentsEntry>) -> ContentsEntry {
    ContentsEntry {
        anchor: percent_encoded(&block.anchor),
        heading: heading(block.kind(), &block.text.name),
        entries,
    }
}
