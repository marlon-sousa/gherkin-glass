use rstest::rstest;

use super::*;
use crate::entities::PageOptions;
use crate::policies::{build_home_page, build_shell_page, build_store, page_frame};
use crate::test_support::{name, parsed_at};

/// The shell of `target`, of a site of one feature, `a`, at its path.
fn shell(target: &BuildTarget) -> (String, String) {
    let parsed = parsed_at("a.feature", "Feature: A\n  Scenario: S\n    Given a\n");
    let options = PageOptions::default();
    let store = build_store(&[&parsed], &options);
    let file = render_shell(
        target,
        &build_shell_page(target, &options, "2.0.0"),
        &page_frame(&options),
        &build_home_page(&store, &options),
        None,
    )
    .expect("ghg's own front page renders");
    let html = String::from_utf8(file.contents.to_vec()).expect("UTF-8");
    (file.address.as_str().to_string(), html)
}

// Rules 241 and 242: a shell names its descriptor and links the assets of the ghg that wrote it.
#[rstest]
#[case::a_site(BuildTarget::Site, "index.html", "site.json", "assets/2.0.0")]
#[case::a_version(
    BuildTarget::Version(name("v1.0.0")),
    "v1.0.0/index.html",
    "versions/v1.0.0.json",
    "../assets/2.0.0"
)]
#[case::a_preview(
    BuildTarget::Preview(name("pr-1")),
    "previews/pr-1/index.html",
    "previews/pr-1.json",
    "../../assets/2.0.0"
)]
fn a_shell_names_its_descriptor_and_links_its_assets(
    #[case] target: BuildTarget,
    #[case] path: &str,
    #[case] descriptor: &str,
    #[case] assets: &str,
) {
    let (address, html) = shell(&target);
    assert_eq!(address, path);
    assert!(
        html.contains(&format!("data-ghg-descriptor=\"{descriptor}\"")),
        "{html}"
    );
    assert!(
        html.contains(&format!("href=\"{assets}/gherkin-glass.css\"")),
        "{html}"
    );
    assert!(
        html.contains(&format!("src=\"{assets}/gherkin-glass.js\"")),
        "{html}"
    );
}

// Rule 241: a site's and a preview's shell hold the front page, and a version's starts empty
// and busy, for the script to draw.
#[rstest]
#[case::a_site(BuildTarget::Site, true)]
#[case::a_version(BuildTarget::Version(name("v1.0.0")), false)]
#[case::a_preview(BuildTarget::Preview(name("pr-1")), true)]
fn only_a_version_s_shell_starts_without_its_front_page(
    #[case] target: BuildTarget,
    #[case] front_page: bool,
) {
    let (_, html) = shell(&target);
    assert_eq!(html.contains("<h1"), front_page, "{html}");
    assert_eq!(html.contains("aria-busy=\"true\""), !front_page, "{html}");
}

#[test]
fn a_preview_s_shell_asks_not_to_be_indexed() {
    let (_, html) = shell(&BuildTarget::Preview(name("pr-1")));
    assert!(html.contains("<meta name=\"robots\""), "{html}");
    let (_, html) = shell(&BuildTarget::Site);
    assert!(!html.contains("<meta name=\"robots\""), "{html}");
}

// Rule 242: the shell carries the page messages, the catalogue's after its marker line, as they
// are, which never close the element that holds them.
#[test]
fn a_shell_carries_the_page_messages_as_they_are() {
    let messages = page_messages();
    assert!(!messages.is_empty());
    assert!(!messages.contains('<'), "{messages}");
    assert!(CATALOGUE_TEXT.contains(PAGE_MESSAGES));
    let (_, html) = shell(&BuildTarget::Site);
    assert!(
        html.contains(&format!(
            "<script type=\"text/plain\" id=\"ghg_messages\">{messages}</script>"
        )),
        "{html}"
    );
}
