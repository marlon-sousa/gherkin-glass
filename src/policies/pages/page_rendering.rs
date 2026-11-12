//! Policy: a page's HTML, a feature's, a folder's or the step sentences', rendered from its askama templates.

mod block_rendering;

use askama::Template;
use block_rendering::render_blocks;

use crate::entities::{Breadcrumb, FolderPage, Page, PageFrame, SentencesPage, TagIndex, TagPage};
use crate::policies::{tag_index_address, tag_pages_language};

#[derive(Template)]
#[template(path = "feature.html")]
struct FeatureTemplate<'a> {
    language: &'a str,
    title: &'a str,
    /// From the page back to the site's root, for the shared assets.
    root: String,
    frame: &'a PageFrame,
    breadcrumb: &'a Breadcrumb,
    page: &'a Page,
    blocks: String,
}

#[derive(Template)]
#[template(path = "folder.html")]
struct FolderTemplate<'a> {
    language: &'a str,
    title: &'a str,
    /// From the page back to the site's root, for the shared assets.
    root: String,
    frame: &'a PageFrame,
    breadcrumb: &'a Breadcrumb,
    folder: &'a FolderPage,
}

/// At the site's root, so the shared assets are beside it.
#[derive(Template)]
#[template(path = "steps.html")]
struct SentencesTemplate<'a> {
    language: &'a str,
    title: &'a str,
    root: &'a str,
    frame: &'a PageFrame,
    breadcrumb: &'a Breadcrumb,
    page: &'a SentencesPage,
}

#[derive(Template)]
#[template(path = "tag_index.html")]
struct TagIndexTemplate<'a> {
    language: &'a str,
    title: &'a str,
    root: String,
    frame: &'a PageFrame,
    breadcrumb: &'a Breadcrumb,
    index: &'a TagIndex,
}

#[derive(Template)]
#[template(path = "tag_page.html")]
struct TagPageTemplate<'a> {
    language: &'a str,
    title: &'a str,
    root: String,
    frame: &'a PageFrame,
    breadcrumb: &'a Breadcrumb,
    page: &'a TagPage,
}

pub(crate) fn render_tag_index(index: &TagIndex) -> String {
    with_final_line_break(render(&TagIndexTemplate {
        language: &tag_pages_language(),
        title: &index.title,
        root: tag_index_address().root(),
        frame: &index.frame,
        breadcrumb: &index.breadcrumb,
        index,
    }))
}

pub(crate) fn render_tag_page(page: &TagPage) -> String {
    with_final_line_break(render(&TagPageTemplate {
        language: &tag_pages_language(),
        title: &page.title,
        root: page.address.root(),
        frame: &page.frame,
        breadcrumb: &page.breadcrumb,
        page,
    }))
}

pub(crate) fn render_page(page: &Page) -> String {
    let template = FeatureTemplate {
        language: &page.language,
        title: &page.title,
        root: page.address.root(),
        frame: &page.frame,
        breadcrumb: &page.breadcrumb,
        page,
        blocks: render_blocks(&page.blocks),
    };
    with_final_line_break(render(&template))
}

pub(crate) fn render_folder_page(folder: &FolderPage) -> String {
    let template = FolderTemplate {
        language: &folder.language,
        title: &folder.title,
        root: folder.address.root(),
        frame: &folder.frame,
        breadcrumb: &folder.breadcrumb,
        folder,
    };
    with_final_line_break(render(&template))
}

pub(crate) fn render_sentences_page(page: &SentencesPage) -> String {
    let template = SentencesTemplate {
        language: &page.language,
        title: &page.title,
        root: "",
        frame: &page.frame,
        breadcrumb: &page.breadcrumb,
        page,
    };
    with_final_line_break(render(&template))
}

/// The templates only write strings, which cannot fail.
#[expect(
    clippy::expect_used,
    reason = "a template that writes only strings renders"
)]
fn render(template: &impl Template) -> String {
    template
        .render()
        .expect("a template that writes only strings renders")
}

fn with_final_line_break(mut html: String) -> String {
    if !html.ends_with('\n') {
        html.push('\n');
    }
    html
}
