//! Policy: the versions page's HTML, from the versions an output folder holds.

use std::fmt::{Display, Formatter, Result as FmtResult};

use askama::Template;
use i18n_embed::LanguageLoader;
use i18n_embed_fl::fl;

use crate::entities::{PageFrame, PageOptions, VersionList, VersionName};
use crate::policies::{CATALOGUE, page_frame};

/// The versions page's address below the output folder.
pub(crate) const VERSIONS_PAGE: &str = "index.html";

#[derive(Template)]
#[template(path = "versions.html")]
struct VersionsTemplate {
    language: String,
    title: String,
    /// The skip link and the themes every page has; the page has no folder tree.
    frame: PageFrame,
    words: Words,
    development: Vec<Entry>,
    releases: Vec<Entry>,
    pre_releases: Vec<Entry>,
}

struct Words {
    title: String,
    development: String,
    releases: String,
    pre_releases: String,
}

/// A version in a list: its name, which is its folder's, and the words of its link.
struct Entry {
    name: VersionName,
    words: String,
}

impl Display for Entry {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        self.name.fmt(formatter)
    }
}

/// The page, in the themes of `options`, those of the build that writes it.
#[expect(
    clippy::expect_used,
    reason = "a template that writes only strings renders"
)]
pub(crate) fn render_versions_page(list: &VersionList, options: &PageOptions) -> String {
    let entries = |names: &[VersionName], words: fn(&VersionName) -> String| {
        names
            .iter()
            .map(|name| Entry {
                name: name.clone(),
                words: words(name),
            })
            .collect()
    };
    let title = fl!(CATALOGUE, "versions-title");
    let template = VersionsTemplate {
        // The page holds interface text only, in the catalogue's language.
        language: CATALOGUE.current_language().to_string(),
        title: title.clone(),
        frame: page_frame(options),
        words: Words {
            title,
            development: fl!(CATALOGUE, "versions-development"),
            releases: fl!(CATALOGUE, "versions-releases"),
            pre_releases: fl!(CATALOGUE, "versions-pre-releases"),
        },
        development: entries(&list.development, |name| {
            fl!(
                CATALOGUE,
                "versions-development-version",
                name = name.as_str()
            )
        }),
        releases: entries(&list.releases, VersionName::to_string),
        pre_releases: entries(&list.pre_releases, VersionName::to_string),
    };
    let mut html = template
        .render()
        .expect("a template that writes only strings renders");
    if !html.ends_with('\n') {
        html.push('\n');
    }
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(names: &[&str]) -> Vec<VersionName> {
        names
            .iter()
            .map(|name| VersionName::new(name).expect("a version name"))
            .collect()
    }

    #[test]
    fn an_empty_list_is_left_out() {
        let html = render_versions_page(
            &VersionList {
                releases: names(&["v1.0.0"]),
                ..VersionList::default()
            },
            &PageOptions::default(),
        );
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
        let html = render_versions_page(
            &VersionList {
                development: names(&["main"]),
                releases: Vec::new(),
                pre_releases: names(&["v2.0.0-rc.1"]),
            },
            &PageOptions::default(),
        );
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
