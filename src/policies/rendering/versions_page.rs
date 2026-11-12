//! Policy: the versions page's HTML, from its node of the site tree.

use askama::Template;

use super::node_rendering::{Render, RenderFile, page};
use super::page_frame::{SkipLink, ThemeControl};
use super::page_head::PageHead;
use super::page_layout::SitePieces;
use crate::entities::{SiteAddress, VersionEntry, VersionsPage, VersionsWords};

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
    words: &'a VersionsWords,
    development: &'a [VersionEntry],
    releases: &'a [VersionEntry],
    pre_releases: &'a [VersionEntry],
}

impl RenderFile for VersionsPage {
    fn address(&self) -> SiteAddress {
        SiteAddress::at_root(VERSIONS_PAGE)
    }

    fn render_file(&self, site: &SitePieces<'_>) -> String {
        page(&VersionsTemplate {
            language: &self.language,
            head: PageHead {
                title: &self.title,
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
            title: &self.title,
            words: &self.words,
            development: &self.development,
            releases: &self.releases,
            pre_releases: &self.pre_releases,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::entities::{PageOptions, SiteNode, VersionName};
    use crate::policies::site_tree;
    use crate::test_support::name;

    /// The versions page of a site whose build makes `version`, once the
    /// output folder holds `others` too.
    fn html(version: &str, others: &[&str]) -> String {
        let options = PageOptions::default();
        let built: BTreeSet<VersionName> = others.iter().copied().map(name).collect();
        let tree = site_tree(&[], &options, Some(&name(version)), &built);
        let Some(SiteNode::VersionsPage(versions)) =
            tree.root_files().next().map(|node| node.value())
        else {
            panic!("a site with versions has its versions page");
        };
        versions.render_file(&SitePieces::of(tree.frame()))
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
