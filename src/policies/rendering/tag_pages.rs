//! Policy: the tag index and each tag's page, their content, each entry of the index, each feature of a tag's page and each of its scenarios rendered from their templates, inside the page's layout, in the language their nodes hold, from their nodes and the nodes below them.

use askama::Template;

use super::breadcrumb::Place;
use super::node_rendering::{NodeOf, Render, RenderFile, pieces, rendered};
use super::page_layout::{PageAround, SitePieces, whole_page};
use crate::entities::{
    SiteAddress, SiteNode, SiteTreeNode, TagFeature, TagIndex, TagIndexEntry, TagPage, TagScenario,
};

#[derive(Template)]
#[template(path = "tag_index.html")]
struct TagIndexTemplate<'a> {
    index: &'a TagIndex,
    /// Its entries, each already rendered.
    entries: Vec<String>,
}

#[derive(Template)]
#[template(path = "tag_index_entry.html")]
struct TagIndexEntryTemplate<'a> {
    entry: &'a TagIndexEntry,
}

#[derive(Template)]
#[template(path = "tag_page.html")]
struct TagPageTemplate<'a> {
    page: &'a TagPage,
    /// Its features, each already rendered.
    features: Vec<String>,
}

#[derive(Template)]
#[template(path = "tag_feature.html")]
struct TagFeatureTemplate<'a> {
    feature: &'a TagFeature,
    /// The words of a feature that writes the tag on no scenario, from its page.
    no_scenario: &'a str,
    /// Its scenarios that carry the tag, each already rendered.
    scenarios: Vec<String>,
}

#[derive(Template)]
#[template(path = "tag_scenario.html")]
struct TagScenarioTemplate<'a> {
    scenario: &'a TagScenario,
    /// Its feature's language, where it differs from the page's.
    language: Option<&'a str>,
}

impl RenderFile for NodeOf<'_, TagIndex> {
    fn address(&self) -> SiteAddress {
        self.value.address.clone()
    }

    fn render_file(&self, site: &SitePieces<'_>) -> String {
        let index = self.value;
        whole_page(
            site,
            &PageAround {
                language: &index.language,
                title: &index.title,
                place: Place::of(self.node, &index.trail),
            },
            rendered(&TagIndexTemplate {
                index,
                entries: pieces(self.node.children()),
            }),
        )
    }
}

impl Render for TagIndexEntry {
    fn render(&self) -> String {
        rendered(&TagIndexEntryTemplate { entry: self })
    }
}

impl RenderFile for NodeOf<'_, TagPage> {
    fn address(&self) -> SiteAddress {
        self.value.address.clone()
    }

    fn render_file(&self, site: &SitePieces<'_>) -> String {
        let page = self.value;
        whole_page(
            site,
            &PageAround {
                language: &page.language,
                title: &page.title,
                place: Place::of(self.node, &page.trail),
            },
            rendered(&TagPageTemplate {
                page,
                features: pieces(self.node.children()),
            }),
        )
    }
}

impl Render for NodeOf<'_, TagFeature> {
    fn render(&self) -> String {
        rendered(&TagFeatureTemplate {
            feature: self.value,
            no_scenario: self
                .node
                .parent()
                .and_then(tag_page)
                .map_or("", |page| page.no_scenario.as_str()),
            scenarios: pieces(self.node.children()),
        })
    }
}

impl Render for NodeOf<'_, TagScenario> {
    fn render(&self) -> String {
        rendered(&TagScenarioTemplate {
            scenario: self.value,
            language: self
                .node
                .parent()
                .and_then(tag_feature)
                .and_then(|feature| feature.language.as_deref()),
        })
    }
}

fn tag_page(node: SiteTreeNode<'_>) -> Option<&TagPage> {
    match node.value() {
        SiteNode::TagPage(page) => Some(page),
        _ => None,
    }
}

fn tag_feature(node: SiteTreeNode<'_>) -> Option<&TagFeature> {
    match node.value() {
        SiteNode::TagFeature(feature) => Some(feature),
        _ => None,
    }
}
