//! Policy: a descriptor of the store as JSON, from what the build stores, the names of its objects and the build's settings.

use serde::Serialize;

use crate::entities::BuildTarget;
use crate::entities::{
    DescriptorFeature, DescriptorSentences, DescriptorTag, OutsideLink, PageOptions, SiteScope,
    SiteStore, TagLink, VersionName,
};
use crate::policies::rendering::{GHG_VERSION, STORE_FORMAT, json_bytes};
use crate::policies::site_title;

#[derive(Serialize)]
struct DescriptorData<'a> {
    format: u32,
    ghg: &'static str,
    version: Option<&'a str>,
    preview: Option<&'a str>,
    title: String,
    only: bool,
    start: StartData,
    features: Vec<FeatureData<'a>>,
    tags: Vec<TagData<'a>>,
    sentences: SentencesData<'a>,
    front_page: Option<&'a str>,
    settings: SettingsData<'a>,
}

#[derive(Serialize)]
struct StartData {
    collapse_rules: bool,
    collapse_scenarios: bool,
    show_tree: bool,
    inline_backgrounds: bool,
}

#[derive(Serialize)]
struct FeatureData<'a> {
    path: &'a str,
    page: &'a str,
    object: &'a str,
    lines: &'a [usize],
    source: Option<&'a str>,
}

#[derive(Serialize)]
struct TagData<'a> {
    tag: &'a str,
    page: &'a str,
    link: Option<LinkData<'a>>,
}

#[derive(Serialize)]
struct LinkData<'a> {
    address: &'a str,
    text: &'a str,
}

#[derive(Serialize)]
struct SentencesData<'a> {
    given: &'a [String],
    when: &'a [String],
    then: &'a [String],
}

#[derive(Serialize)]
struct SettingsData<'a> {
    title: Option<&'a str>,
    source_link: Option<&'a str>,
    tag_links: Vec<TagLinkData<'a>>,
    accent_light: String,
    accent_dark: String,
    collapse_rules: bool,
    collapse_scenarios: bool,
    show_tree: bool,
    inline_backgrounds: bool,
}

#[derive(Serialize)]
struct TagLinkData<'a> {
    pattern: &'a str,
    address: &'a str,
    text: &'a str,
}

/// What a descriptor is written from besides the store: what the build writes, the settings of
/// every page, and the front page's template as it was read, if any.
pub(crate) struct DescriptorSettings<'a> {
    pub(crate) target: &'a BuildTarget,
    pub(crate) options: &'a PageOptions,
    pub(crate) front_page: Option<&'a str>,
}

/// The compact JSON of the descriptor of `store`, whose features' objects are named, in their
/// order, by `objects`.
pub(super) fn descriptor_json(
    store: &SiteStore,
    objects: &[String],
    settings: &DescriptorSettings<'_>,
) -> Vec<u8> {
    let options = settings.options;
    let features = store
        .features
        .iter()
        .zip(objects)
        .map(|(feature, object)| feature_data(feature, object))
        .collect();
    json_bytes(&DescriptorData {
        format: STORE_FORMAT,
        ghg: GHG_VERSION,
        version: settings.target.version().map(VersionName::as_str),
        preview: settings.target.preview().map(VersionName::as_str),
        title: site_title(options),
        only: options.scope == SiteScope::OnlyListed,
        start: StartData {
            collapse_rules: options.collapse_rules,
            collapse_scenarios: options.collapse_scenarios,
            show_tree: options.show_tree,
            inline_backgrounds: options.inline_backgrounds,
        },
        features,
        tags: store.tags.iter().map(tag_data).collect(),
        sentences: sentences_data(&store.sentences),
        // A version's front page is the versions page's.
        front_page: match settings.target {
            BuildTarget::Version(_) => None,
            BuildTarget::Site | BuildTarget::Preview(_) => settings.front_page,
        },
        settings: settings_data(options),
    })
}

fn feature_data<'a>(feature: &'a DescriptorFeature, object: &'a str) -> FeatureData<'a> {
    FeatureData {
        path: feature.path.as_str(),
        page: &feature.page,
        object,
        lines: &feature.lines,
        source: feature.source.as_deref(),
    }
}

fn tag_data(tag: &DescriptorTag) -> TagData<'_> {
    TagData {
        tag: &tag.tag,
        page: &tag.page,
        link: tag.link.as_ref().map(link_data),
    }
}

fn link_data(link: &OutsideLink) -> LinkData<'_> {
    LinkData {
        address: &link.address,
        text: &link.text,
    }
}

fn sentences_data(sentences: &DescriptorSentences) -> SentencesData<'_> {
    SentencesData {
        given: &sentences.given,
        when: &sentences.when,
        then: &sentences.then,
    }
}

fn settings_data(options: &PageOptions) -> SettingsData<'_> {
    SettingsData {
        title: options.title.as_deref(),
        source_link: options.source_link.as_deref(),
        tag_links: options.tag_links.iter().map(tag_link_data).collect(),
        accent_light: options.accents.light.to_string(),
        accent_dark: options.accents.dark.to_string(),
        collapse_rules: options.collapse_rules,
        collapse_scenarios: options.collapse_scenarios,
        show_tree: options.show_tree,
        inline_backgrounds: options.inline_backgrounds,
    }
}

fn tag_link_data(link: &TagLink) -> TagLinkData<'_> {
    TagLinkData {
        pattern: &link.source,
        address: &link.address,
        text: &link.text,
    }
}
