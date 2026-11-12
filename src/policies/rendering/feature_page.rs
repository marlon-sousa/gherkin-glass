//! Policy: a feature's page, its content rendered from its template inside the page's layout, from the page node and the blocks below it.

use askama::Template;

use super::breadcrumb::Place;
use super::node_rendering::{Render, RenderFile, pieces, rendered};
use super::page_layout::{PageAround, SitePieces, whole_page};
use crate::entities::{ContentsEntry, Page, SiteAddress, SiteFeaturePage, SiteTreeNode};

#[derive(Template)]
#[template(path = "feature_page.html")]
struct FeatureTemplate<'a> {
    page: &'a Page,
    contents: Vec<ContentsEntry>,
    /// Its blocks, already rendered; empty for a page with none.
    blocks: String,
    /// Whether a scenario starts with a background's steps, so that the
    /// checkbox that switches how they show can act.
    backgrounds_enabled: bool,
}

impl RenderFile for SiteFeaturePage<'_> {
    fn address(&self) -> SiteAddress {
        self.page().address.clone()
    }

    fn render_file(&self, site: &SitePieces<'_>) -> String {
        let page = self.page();
        whole_page(
            site,
            &PageAround {
                language: &page.language,
                title: &page.title,
                place: Place::of(self.node(), &page.trail),
            },
            rendered(&FeatureTemplate {
                page,
                contents: self.contents(),
                blocks: pieces(self.blocks()).join("\n"),
                backgrounds_enabled: self.scenarios().any(starts_with_a_background),
            }),
        )
    }
}

fn starts_with_a_background(scenario: SiteTreeNode<'_>) -> bool {
    !scenario.backgrounds().is_empty()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::entities::PageOptions;
    use crate::test_support::{feature_pages, site_of};

    #[rstest]
    #[case::without_steps(
        "Feature: F\n  Background:\n    Only words.\n\n  Scenario: S\n    Given x\n",
        false
    )]
    #[case::with_no_scenario("Feature: F\n  Background:\n    Given x\n", false)]
    #[case::beside_a_scenario(
        "Feature: F\n  Background:\n    Given x\n  Scenario: S\n    Given y\n",
        true
    )]
    #[case::in_a_rule(
        "Feature: F\n  Rule: R\n    Background:\n      Given x\n    Scenario: S\n      Given y\n",
        true
    )]
    fn the_option_acts_once_a_scenario_starts_with_a_background_s_steps(
        #[case] text: &str,
        #[case] enabled: bool,
    ) {
        let site = site_of(&[("f.feature", text)], &PageOptions::default());
        let [page] = feature_pages(&site)[..] else {
            panic!("one page");
        };
        let html = page.render_file(&SitePieces::of(site.frame()));
        assert!(html.contains("ghg-inline-backgrounds"), "{html}");
        assert_eq!(!html.contains(" disabled>"), enabled, "{html}");
    }
}
