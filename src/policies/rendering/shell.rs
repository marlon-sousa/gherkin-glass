//! Policy: a shell, the one file from which the script draws every page of a site without versions, of a version or of a preview, its `main` holding the home page of a site and of a preview, rendered from its template.

use askama::Template;

use super::home_page::home_main;
use super::node_rendering::{Render, page};
use super::page_frame::{
    DiskNotice, PageFooter, ShellOnDisk, ShellOptions, SkipLink, TreeLandmark,
};
use super::page_head::PageHead;
use super::search_box::SearchBox;
use crate::entities::{
    BuildTarget, FrontPageTemplate, HomePage, PageFrame, ShellPage, SiteAddress, SiteFile,
};
use crate::errors::FrontPageError;
use crate::policies::rendering::shell_path;

/// The catalogue, whose messages after [`PAGE_MESSAGES`] the script draws pages with.
const CATALOGUE_TEXT: &str = include_str!("../../../messages/en/gherkin-glass.ftl");

/// The line of the catalogue after which every message is a page's.
const PAGE_MESSAGES: &str = "### ghg:page-messages\n";

/// The messages of the catalogue the script draws pages with, which every shell carries.
fn page_messages() -> &'static str {
    CATALOGUE_TEXT
        .split_once(PAGE_MESSAGES)
        .map_or("", |(_, messages)| messages)
}

#[derive(Template)]
#[template(path = "shell.html")]
struct ShellTemplate<'a> {
    descriptor: &'a str,
    head: String,
    messages: &'a str,
    disk_notice: String,
    skip_link: String,
    options: String,
    tree: String,
    breadcrumb: &'a str,
    search: String,
    busy: bool,
    only_note: Option<&'a str>,
    main: String,
    footer: String,
    needs_script: &'a str,
}

/// The shell of `target` that `shell` describes, in the frame `frame`, its `main` holding
/// `home`, when it holds the front page, written from `template`, or from ghg's own without one.
pub(crate) fn render_shell(
    target: &BuildTarget,
    shell: &ShellPage,
    frame: &PageFrame,
    home: &HomePage,
    template: Option<&FrontPageTemplate>,
) -> Result<SiteFile, FrontPageError> {
    let (title, main) = if shell.holds_front_page {
        let front = home_main(home, template)?;
        (front.title, front.main)
    } else {
        (shell.title.clone(), String::new())
    };
    let html = page(&ShellTemplate {
        descriptor: &shell.descriptor,
        head: PageHead {
            title: &title,
            description: Some(&shell.description),
            unindexed: shell.unindexed,
            assets: &shell.assets,
            stylesheet: &frame.accents.render(),
        }
        .render(),
        messages: page_messages(),
        disk_notice: DiskNotice {
            frame,
            shell: Some(if shell.holds_front_page {
                ShellOnDisk::AllButTheHomePage
            } else {
                ShellOnDisk::EveryPage
            }),
        }
        .render(),
        skip_link: SkipLink {
            frame,
            href: "##ghg_main",
        }
        .render(),
        options: ShellOptions { frame }.render(),
        tree: TreeLandmark { frame }.render(),
        breadcrumb: &frame.breadcrumb,
        search: SearchBox {
            words: &frame.search,
        }
        .render(),
        busy: !shell.holds_front_page,
        // A version's `main` starts empty, so that the script draws its home page, the note in it.
        only_note: frame
            .only_note
            .as_deref()
            .filter(|_| shell.holds_front_page),
        main,
        footer: PageFooter { frame }.render(),
        needs_script: &frame.needs_script,
    });
    Ok(SiteFile::text(
        SiteAddress::at_root(&shell_path(target)),
        html,
    ))
}

#[cfg(test)]
mod tests;
