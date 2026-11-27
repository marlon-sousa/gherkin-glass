//! Policy: the versions page's HTML, a front page, its lists and each of their entries rendered from their templates.

use askama::Template;

use super::front_page::front_page;
use super::node_rendering::{Render, page, rendered};
use super::page_frame::{DiskNotice, PageFooter, SkipLink, ThemeControl};
use super::page_head::PageHead;
use crate::entities::{
    FrontPageTemplate, HeadingLevel, PageFrame, SiteAddress, SiteFile, VersionEntry, VersionName,
    VersionsPageContent, VersionsPageRecord,
};
use crate::errors::FrontPageError;
use crate::policies::build_versions_page;
use crate::policies::rendering::{ROOT_PAGE, VersionsPageWrite, assets_folder};

#[derive(Template)]
#[template(path = "versions_page.html")]
struct VersionsTemplate<'a> {
    language: &'a str,
    head: String,
    disk_notice: String,
    skip_link: String,
    theme_control: String,
    /// Its main content, already rendered.
    main: String,
    footer: String,
}

/// The lists that hold a version, each already rendered.
#[derive(Template)]
#[template(path = "versions_lists.html")]
struct VersionsListsTemplate {
    lists: Vec<String>,
}

#[derive(Template)]
#[template(path = "versions_list.html")]
struct VersionsListTemplate<'a> {
    heading: &'a str,
    level: HeadingLevel,
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

/// The versions page listing `names`, written with `record`, in the frame `frame`, from
/// `template`, or from ghg's own without one.
pub(crate) fn versions_page_write(
    record: VersionsPageRecord,
    names: Vec<VersionName>,
    frame: &PageFrame,
    template: Option<&FrontPageTemplate>,
) -> Result<VersionsPageWrite, FrontPageError> {
    let content = build_versions_page(names, &record.title);
    let page = render_versions_page(&content, frame, &record.ghg, template)?;
    Ok(VersionsPageWrite { record, page })
}

/// The versions page of `content`, in the frame `frame`, linking the assets of the ghg `ghg`,
/// written from `template`, or from ghg's own without one.
pub(crate) fn render_versions_page(
    content: &VersionsPageContent,
    frame: &PageFrame,
    ghg: &str,
    template: Option<&FrontPageTemplate>,
) -> Result<SiteFile, FrontPageError> {
    let words = &content.page.words;
    let lists = [
        (
            &words.development,
            "development",
            &content.development,
            false,
        ),
        (&words.releases, "releases", &content.releases, false),
        (
            &words.pre_releases,
            "pre-releases",
            &content.pre_releases,
            true,
        ),
    ];
    let front = front_page(template, &content.page.heading, |level| {
        rendered(&VersionsListsTemplate {
            lists: lists
                .iter()
                .filter(|(_, _, entries, _)| !entries.is_empty())
                .map(|(heading, id, entries, collapsed)| {
                    rendered(&VersionsListTemplate {
                        heading,
                        level,
                        id,
                        collapsed: *collapsed,
                        entries: entries.iter().map(Render::render).collect(),
                    })
                })
                .collect(),
        })
    })?;
    let assets = assets_folder(ghg);
    let html = page(&VersionsTemplate {
        language: &content.page.language,
        head: PageHead {
            title: match template {
                Some(_) => &front.title,
                None => &content.page.title,
            },
            description: None,
            unindexed: false,
            assets: &assets,
            stylesheet: &frame.accents.render(),
        }
        .render(),
        disk_notice: DiskNotice { frame, shell: None }.render(),
        skip_link: SkipLink {
            frame,
            href: "#ghg_main",
        }
        .render(),
        theme_control: ThemeControl { frame }.render(),
        main: front.main,
        footer: PageFooter { frame }.render(),
    });
    Ok(SiteFile::text(SiteAddress::at_root(ROOT_PAGE), html))
}

impl Render for VersionEntry {
    fn render(&self) -> String {
        rendered(&VersionEntryTemplate { entry: self })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::PageOptions;
    use crate::policies::{build_versions_page, page_frame};
    use crate::test_support::name;

    /// The versions page of `versions`.
    fn html(versions: &[&str]) -> String {
        let content = build_versions_page(versions.iter().copied().map(name), "Feature files");
        let frame = page_frame(&PageOptions::default());
        let page = render_versions_page(&content, &frame, "2.0.0", None)
            .expect("ghg's own front page renders");
        assert_eq!(page.address.as_str(), "index.html");
        String::from_utf8(page.contents.to_vec()).expect("UTF-8")
    }

    #[test]
    fn an_empty_list_is_left_out() {
        let html = html(&["v1.0.0"]);
        assert!(html.contains("<h2 id=\"releases\">Releases</h2>"), "{html}");
        assert!(!html.contains("id=\"development\""), "{html}");
        assert!(!html.contains("id=\"pre-releases\""), "{html}");
        assert!(html.contains("<html lang=\"en\">"), "{html}");
        assert!(
            html.contains("<title>Versions, Feature files</title>"),
            "{html}"
        );
        assert!(
            html.contains("href=\"assets/2.0.0/gherkin-glass.css\""),
            "{html}"
        );
        assert!(
            html.contains("<a href=\"v1.0.0/index.html\">v1.0.0</a>"),
            "{html}"
        );
    }

    #[test]
    fn a_development_version_says_so_and_pre_releases_start_collapsed() {
        let html = html(&["main", "v2.0.0-rc.1"]);
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
