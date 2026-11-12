//! Policy: the tag index and each tag's page, their content rendered from their templates inside the page's layout, in the language their nodes hold, from their nodes and the nodes below them.

use askama::Template;

use super::node_rendering::{PageNode, Render, RenderFile, rendered};
use super::page_layout::{PageAround, SitePieces, whole_page};
use crate::entities::{
    SiteAddress, SiteNode, SiteTreeNode, TagFeature, TagIndex, TagIndexEntry, TagPage, TagScenario,
};

#[derive(Template)]
#[template(path = "tag_index.html")]
struct TagIndexTemplate<'a> {
    index: &'a TagIndex,
    entries: Vec<&'a TagIndexEntry>,
}

#[derive(Template)]
#[template(path = "tag_page.html")]
struct TagPageTemplate<'a> {
    page: &'a TagPage,
    features: Vec<FeatureOnTagPage<'a>>,
}

/// A feature on a tag's page, with its scenarios that carry the tag.
struct FeatureOnTagPage<'a> {
    feature: &'a TagFeature,
    scenarios: Vec<&'a TagScenario>,
}

impl RenderFile for PageNode<'_, TagIndex> {
    fn address(&self) -> SiteAddress {
        self.page.address.clone()
    }

    fn render_file(&self, site: &SitePieces<'_>) -> String {
        let index = self.page;
        whole_page(
            site,
            &PageAround {
                language: &index.language,
                title: &index.title,
                root: index.address.root(),
                breadcrumb: &index.breadcrumb,
            },
            rendered(&TagIndexTemplate {
                index,
                entries: self
                    .node
                    .children()
                    .filter_map(|child| match child.value() {
                        SiteNode::TagIndexEntry(entry) => Some(entry),
                        _ => None,
                    })
                    .collect(),
            }),
        )
    }
}

impl RenderFile for PageNode<'_, TagPage> {
    fn address(&self) -> SiteAddress {
        self.page.address.clone()
    }

    fn render_file(&self, site: &SitePieces<'_>) -> String {
        let page = self.page;
        whole_page(
            site,
            &PageAround {
                language: &page.language,
                title: &page.title,
                root: page.address.root(),
                breadcrumb: &page.breadcrumb,
            },
            rendered(&TagPageTemplate {
                page,
                features: self
                    .node
                    .children()
                    .filter_map(feature_on_tag_page)
                    .collect(),
            }),
        )
    }
}

/// The feature of `node`, when it is one, with its scenarios below it.
fn feature_on_tag_page(node: SiteTreeNode<'_>) -> Option<FeatureOnTagPage<'_>> {
    let SiteNode::TagFeature(feature) = node.value() else {
        return None;
    };
    Some(FeatureOnTagPage {
        feature,
        scenarios: node
            .children()
            .filter_map(|child| match child.value() {
                SiteNode::TagScenario(scenario) => Some(scenario),
                _ => None,
            })
            .collect(),
    })
}
