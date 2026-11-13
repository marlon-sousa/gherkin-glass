//! Policy: the page of step sentences, its content, each group, each sentence and each use rendered from their templates, inside the page's layout, from the page's node and the nodes below it.

use askama::Template;

use super::breadcrumb::Place;
use super::node_rendering::{NodeOf, Render, RenderFile, pieces, rendered};
use super::page_layout::{PageAround, SitePieces, whole_page};
use crate::entities::{GroupView, SentenceView, SentencesPage, SiteAddress, UseView};
use crate::errors::FrontPageError;
use crate::policies::SENTENCES_PAGE;

#[derive(Template)]
#[template(path = "sentences_page.html")]
struct SentencesTemplate<'a> {
    page: &'a SentencesPage,
    /// Its groups, each already rendered.
    groups: Vec<String>,
}

#[derive(Template)]
#[template(path = "sentences_group.html")]
struct GroupTemplate<'a> {
    group: &'a GroupView,
    /// Its sentences, each already rendered.
    sentences: Vec<String>,
}

#[derive(Template)]
#[template(path = "sentence.html")]
struct SentenceTemplate<'a> {
    sentence: &'a SentenceView,
    /// Its uses, each already rendered.
    uses: Vec<String>,
}

#[derive(Template)]
#[template(path = "sentence_use.html")]
struct UseTemplate<'a> {
    found: &'a UseView,
}

impl RenderFile for NodeOf<'_, SentencesPage> {
    fn address(&self) -> SiteAddress {
        SiteAddress::at_root(SENTENCES_PAGE)
    }

    fn render_file(&self, site: &SitePieces<'_>) -> Result<String, FrontPageError> {
        let page = self.value;
        Ok(whole_page(
            site,
            &PageAround {
                language: &page.language,
                title: &page.title,
                place: Place::of(self.node, &page.trail),
            },
            rendered(&SentencesTemplate {
                page,
                groups: pieces(self.node.children()),
            }),
        ))
    }
}

impl Render for NodeOf<'_, GroupView> {
    fn render(&self) -> String {
        rendered(&GroupTemplate {
            group: self.value,
            sentences: pieces(self.node.children()),
        })
    }
}

impl Render for NodeOf<'_, SentenceView> {
    fn render(&self) -> String {
        rendered(&SentenceTemplate {
            sentence: self.value,
            uses: pieces(self.node.children()),
        })
    }
}

impl Render for UseView {
    fn render(&self) -> String {
        rendered(&UseTemplate { found: self })
    }
}
