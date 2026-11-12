//! Policy: the page of step sentences, its content rendered from its template inside the page's layout, from its node and the nodes below it.

use askama::Template;

use super::node_rendering::{PageNode, RenderFile, rendered};
use super::page_layout::{PageAround, SitePieces, whole_page};
use crate::entities::{
    GroupView, SentenceView, SentencesPage, SiteAddress, SiteNode, SiteTreeNode, UseView,
};
use crate::policies::SENTENCES_PAGE;

#[derive(Template)]
#[template(path = "sentences_page.html")]
struct SentencesTemplate<'a> {
    page: &'a SentencesPage,
    groups: Vec<GroupOnPage<'a>>,
}

/// A group of the page, with its sentences.
struct GroupOnPage<'a> {
    group: &'a GroupView,
    sentences: Vec<SentenceOnPage<'a>>,
}

/// A sentence of the page, with its uses.
struct SentenceOnPage<'a> {
    sentence: &'a SentenceView,
    uses: Vec<&'a UseView>,
}

impl RenderFile for PageNode<'_, SentencesPage> {
    fn address(&self) -> SiteAddress {
        SiteAddress::at_root(SENTENCES_PAGE)
    }

    fn render_file(&self, site: &SitePieces<'_>) -> String {
        let page = self.page;
        whole_page(
            site,
            &PageAround {
                language: &page.language,
                title: &page.title,
                root: self.address().root(),
                breadcrumb: &page.breadcrumb,
            },
            rendered(&SentencesTemplate {
                page,
                groups: self.node.children().filter_map(group_on_page).collect(),
            }),
        )
    }
}

/// The group of `node`, when it is one, with its sentences below it.
fn group_on_page(node: SiteTreeNode<'_>) -> Option<GroupOnPage<'_>> {
    let SiteNode::SentencesGroup(group) = node.value() else {
        return None;
    };
    Some(GroupOnPage {
        group,
        sentences: node.children().filter_map(sentence_on_page).collect(),
    })
}

/// The sentence of `node`, when it is one, with its uses below it.
fn sentence_on_page(node: SiteTreeNode<'_>) -> Option<SentenceOnPage<'_>> {
    let SiteNode::Sentence(sentence) = node.value() else {
        return None;
    };
    Some(SentenceOnPage {
        sentence,
        uses: node
            .children()
            .filter_map(|child| match child.value() {
                SiteNode::SentenceUse(found) => Some(found),
                _ => None,
            })
            .collect(),
    })
}
