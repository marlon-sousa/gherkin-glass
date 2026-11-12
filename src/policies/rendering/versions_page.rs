//! Policy: the versions page's HTML, its lists and each of their entries rendered from their templates, from its node of the site tree and the nodes below it.

use askama::Template;

use super::node_rendering::{NodeOf, Render, RenderFile, page, pieces, rendered};
use super::page_frame::{SkipLink, ThemeControl};
use super::page_head::PageHead;
use super::page_layout::SitePieces;
use crate::entities::{
    SiteAddress, SiteNode, SiteTreeNode, VersionEntry, VersionsList, VersionsPage, VersionsWords,
};

/// The versions page's address below the output folder.
const VERSIONS_PAGE: &str = "index.html";

#[derive(Template)]
#[template(path = "versions_page.html")]
struct VersionsTemplate<'a> {
    language: &'a str,
    head: String,
    skip_link: String,
    theme_control: String,
    title: &'a str,
    /// Its lists that hold a version, each already rendered.
    lists: Vec<String>,
}

#[derive(Template)]
#[template(path = "versions_list.html")]
struct VersionsListTemplate<'a> {
    heading: &'a str,
    id: &'a str,
    /// Whether the list starts collapsed, as the pre-releases do.
    collapsed: bool,
    /// Its entries, each already rendered.
    entries: Vec<String>,
}

#[derive(Template)]
#[template(path = "version_entry.html")]
struct VersionEntryTemplate<'a> {
    entry: &'a VersionEntry,
}

impl RenderFile for NodeOf<'_, VersionsPage> {
    fn address(&self) -> SiteAddress {
        SiteAddress::at_root(VERSIONS_PAGE)
    }

    fn render_file(&self, site: &SitePieces<'_>) -> String {
        let versions = self.value;
        page(&VersionsTemplate {
            language: &versions.language,
            head: PageHead {
                title: &versions.title,
                root: "",
                stylesheet: site.stylesheet(),
            }
            .render(),
            skip_link: SkipLink {
                frame: site.page_frame(),
            }
            .render(),
            theme_control: ThemeControl {
                frame: site.page_frame(),
            }
            .render(),
            title: &versions.title,
            lists: pieces(self.node.children().filter(holds_a_version)),
        })
    }
}

impl Render for NodeOf<'_, VersionsList> {
    fn render(&self) -> String {
        let words = self.node.parent().and_then(versions_words);
        let (heading, id) = match self.value {
            VersionsList::Development => (words.map(|words| &words.development), "development"),
            VersionsList::Releases => (words.map(|words| &words.releases), "releases"),
            VersionsList::PreReleases => (words.map(|words| &words.pre_releases), "pre-releases"),
        };
        rendered(&VersionsListTemplate {
            heading: heading.map_or("", String::as_str),
            id,
            collapsed: *self.value == VersionsList::PreReleases,
            entries: pieces(self.node.children()),
        })
    }
}

impl Render for VersionEntry {
    fn render(&self) -> String {
        rendered(&VersionEntryTemplate { entry: self })
    }
}

fn holds_a_version(list: &SiteTreeNode<'_>) -> bool {
    list.children().next().is_some()
}

/// The headings of the lists of `node`, when it is the versions page.
fn versions_words(node: SiteTreeNode<'_>) -> Option<&VersionsWords> {
    match node.value() {
        SiteNode::VersionsPage(page) => Some(&page.words),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::entities::{PageOptions, VersionName};
    use crate::policies::site_tree;
    use crate::test_support::name;

    /// The versions page of a site whose build makes `version`, once the
    /// output folder holds `others` too.
    fn html(version: &str, others: &[&str]) -> String {
        let options = PageOptions::default();
        let built: BTreeSet<VersionName> = others.iter().copied().map(name).collect();
        let tree = site_tree(&[], None, &options, Some(&name(version)), &built);
        let Some(node) = tree.root_files().next() else {
            panic!("a site with versions has its versions page");
        };
        let SiteNode::VersionsPage(versions) = node.value() else {
            panic!("the versions page first");
        };
        NodeOf {
            value: versions,
            node,
        }
        .render_file(&SitePieces::of(tree.frame()))
    }

    #[test]
    fn an_empty_list_is_left_out() {
        let html = html("v1.0.0", &[]);
        assert!(html.contains("<h2 id=\"releases\">Releases</h2>"), "{html}");
        assert!(!html.contains("id=\"development\""), "{html}");
        assert!(!html.contains("id=\"pre-releases\""), "{html}");
        assert!(html.contains("<html lang=\"en\">"), "{html}");
        assert!(
            html.contains("<a href=\"v1.0.0/index.html\">v1.0.0</a>"),
            "{html}"
        );
    }

    #[test]
    fn a_development_version_says_so_and_pre_releases_start_collapsed() {
        let html = html("main", &["v2.0.0-rc.1"]);
        assert!(
            html.contains("<a href=\"main/index.html\">main, a development version</a>"),
            "{html}"
        );
        assert!(
            html.contains("aria-expanded=\"false\">Pre-releases</button></h2>\n<div class=\"ghg-content\" hidden>"),
            "{html}"
        );
    }
}
