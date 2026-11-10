//! Policy: a page's HTML, rendered from its askama templates.

use askama::Template;

use crate::entities::{Block, Page};
use crate::policies::root_from;

#[derive(Template)]
#[template(path = "feature.html")]
struct FeatureTemplate<'a> {
    language: &'a str,
    title: &'a str,
    /// From the page back to the site's root, for the shared assets.
    root: String,
    page: &'a Page,
    /// Rendered already, since a block holds blocks.
    blocks: String,
}

#[derive(Template)]
#[template(path = "block.html")]
struct BlockTemplate<'a> {
    block: &'a Block,
    blocks: String,
}

pub(crate) fn render_page(page: &Page) -> String {
    let template = FeatureTemplate {
        language: &page.language,
        title: &page.title,
        root: root_from(&page.address),
        page,
        blocks: render_blocks(&page.blocks),
    };
    with_final_line_break(render(&template))
}

fn render_blocks(blocks: &[Block]) -> String {
    blocks
        .iter()
        .map(|block| {
            render(&BlockTemplate {
                block,
                blocks: render_blocks(&block.blocks),
            })
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The templates only write strings, which cannot fail.
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
