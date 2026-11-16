//! Rule 203: markup in a feature file reaches every page as text, checked by
//! building `tests/fixtures/hostile` and parsing each page it writes as a
//! browser does, with `scraper`.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs::{read_dir, read_to_string, remove_dir_all};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use rstest::rstest;
use scraper::{ElementRef, Html, Selector};

/// Every element the pages' templates and renderers write.
const ELEMENTS: [&str; 41] = [
    "a",
    "bdi",
    "body",
    "br",
    "button",
    "dialog",
    "div",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "head",
    "html",
    "input",
    "label",
    "legend",
    "li",
    "link",
    "main",
    "meta",
    "nav",
    "ol",
    "p",
    "pre",
    "script",
    "span",
    "style",
    "table",
    "tbody",
    "td",
    "th",
    "thead",
    "title",
    "tr",
    "ul",
];

/// Every attribute the pages' templates and renderers write.
const ATTRIBUTES: [&str; 65] = [
    "action",
    "aria-autocomplete",
    "aria-controls",
    "aria-current",
    "aria-describedby",
    "aria-disabled",
    "aria-expanded",
    "aria-haspopup",
    "aria-label",
    "aria-labelledby",
    "autocomplete",
    "charset",
    "checked",
    "class",
    "content",
    "data-all-results",
    "data-background",
    "data-background-in-rule",
    "data-count",
    "data-count-limited",
    "data-examples",
    "data-examples-in-rule",
    "data-expanded-when-unchecked",
    "data-failed",
    "data-feature",
    "data-heading",
    "data-loading",
    "data-matching-cells",
    "data-matching-steps",
    "data-matching-tags",
    "data-no-name",
    "data-none",
    "data-numbered-examples",
    "data-root",
    "data-rule",
    "data-scenario",
    "data-scenario-in-rule",
    "data-scenario-outline",
    "data-scenario-outline-in-rule",
    "data-separator",
    "data-shown",
    "data-title",
    "data-unnamed-background",
    "data-unnamed-feature",
    "data-unnamed-rule",
    "data-unnamed-scenario",
    "data-unnamed-scenario-outline",
    "data-within",
    "disabled",
    "enterkeyhint",
    "for",
    "hidden",
    "href",
    "id",
    "lang",
    "method",
    "name",
    "rel",
    "role",
    "scope",
    "spellcheck",
    "src",
    "tabindex",
    "type",
    "value",
];

/// The elements ghg writes only in a page's head.
const HEAD_ONLY: [&str; 4] = ["link", "meta", "style", "title"];

/// The scripts every page has: the theme's line, the line that opens the
/// dialog for a page opened from disk, and the pages' script.
const SCRIPTS_OF_A_PAGE: usize = 3;

/// The site `tests/fixtures/hostile` builds into, built once for every test.
fn site() -> &'static Path {
    static SITE: OnceLock<PathBuf> = OnceLock::new();
    SITE.get_or_init(|| {
        let output = Path::new(env!("CARGO_TARGET_TMPDIR")).join("hostile");
        let _ = remove_dir_all(&output);
        let built = Command::new(env!("CARGO_BIN_EXE_ghg"))
            .args(["build", "--output"])
            .arg(&output)
            .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hostile"))
            .output()
            .expect("the command runs");
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
        output
    })
}

/// Every page of the site, by its `/`-joined path in it.
fn pages() -> Vec<String> {
    let mut found = Vec::new();
    let mut folders = vec![(site().to_path_buf(), String::new())];
    while let Some((folder, prefix)) = folders.pop() {
        for entry in read_dir(&folder).expect("the folder is listed") {
            let entry = entry.expect("the folder is listed");
            let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
            if entry.path().is_dir() {
                folders.push((entry.path(), format!("{name}/")));
            } else if name.ends_with(".html") {
                found.push(name);
            }
        }
    }
    found.sort();
    found
}

fn parsed(page: &str) -> Html {
    Html::parse_document(&read_to_string(site().join(page)).expect("the page is read"))
}

fn elements(html: &Html) -> impl Iterator<Item = ElementRef<'_>> {
    html.root_element()
        .descendants()
        .filter_map(ElementRef::wrap)
}

/// The text of the page's title and body as a reader gets it, every run of
/// white space read as one space.
fn page_text(html: &Html) -> String {
    let parts = Selector::parse("title, body").expect("the selector is valid");
    let text: Vec<String> = html
        .select(&parts)
        .map(|part| part.text().collect())
        .collect();
    text.join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_head(element: ElementRef<'_>) -> bool {
    element.value().name() == "head"
}

fn is_outside_head(element: &ElementRef<'_>) -> bool {
    !element
        .ancestors()
        .filter_map(ElementRef::wrap)
        .any(is_head)
}

#[test]
fn the_hostile_fixture_builds_a_page_for_every_feature_folder_and_tag() {
    assert_eq!(
        pages(),
        [
            "Q&A's notes/answers.html",
            "Q&A's notes/index.html",
            "hostile.html",
            "index.html",
            "search.html",
            "steps.html",
            "tags/b-feature-tag-b.html",
            "tags/em-outline-tag-em.html",
            "tags/i-scenario-tag-i.html",
            "tags/index.html",
            "tags/s-examples-tag-s.html",
            "tags/ticket-script-alert-ticket-script.html",
        ]
    );
}

// Rule 203: no element or attribute but those ghg writes.
#[test]
fn every_page_holds_only_the_elements_and_attributes_ghg_writes() {
    for page in pages() {
        let html = parsed(&page);
        for element in elements(&html) {
            let name = element.value().name();
            assert!(ELEMENTS.contains(&name), "{page}: an element {name}");
            for (attribute, value) in element.value().attrs() {
                assert!(
                    ATTRIBUTES.contains(&attribute),
                    "{page}: an attribute {attribute} on {name}"
                );
                assert!(
                    !value.trim_start().to_lowercase().starts_with("javascript:"),
                    "{page}: {attribute}={value:?} on {name}"
                );
            }
        }
    }
}

// Rule 203: what only a page's head may hold is only in its head.
#[test]
fn every_page_holds_its_title_style_links_and_metadata_only_in_its_head() {
    for page in pages() {
        let html = parsed(&page);
        for element in elements(&html).filter(is_outside_head) {
            let name = element.value().name();
            assert!(
                !HEAD_ONLY.contains(&name),
                "{page}: an element {name} outside the head"
            );
        }
    }
}

// Rule 203: the only scripts are ghg's own.
#[test]
fn every_page_runs_only_ghg_s_own_scripts() {
    let script = Selector::parse("script").expect("the selector is valid");
    for page in pages() {
        let scripts = parsed(&page).select(&script).count();
        assert_eq!(scripts, SCRIPTS_OF_A_PAGE, "{page}");
    }
}

// Rule 203: the HTML writes the text with character references, and the
// addresses made from it percent-encoded.
#[rstest]
#[case::the_feature_s_heading(
    "hostile.html",
    "<h1>Feature: <bdi>Hostile &#60;script&#62;alert(&#34;feature&#34;)&#60;/script&#62; &#38; &#39;friends&#39;</bdi></h1>"
)]
#[case::a_tag_s_link_to_another_site(
    "hostile.html",
    "href=\"https://example.com/tickets/%22%3E%3Cscript%3Ealert%28%27ticket%27%29%3C/script%3E\""
)]
#[case::the_source_link_of_a_file_in_a_folder_with_an_ampersand_and_an_apostrophe(
    "Q&A's notes/answers.html",
    "href=\"https://example.com/repo/blob/main/features/Q%26A%27s%20notes/answers.feature#L1\""
)]
fn the_html_writes_the_text_escaped(#[case] page: &str, #[case] written: &str) {
    let html = read_to_string(site().join(page)).expect("the page is read");
    assert!(html.contains(written), "{page} does not hold {written}");
}

// Rule 203: the text reads as written in the feature files.
#[rstest]
#[case::the_feature_page(
    "hostile.html",
    &[
        "Feature: Hostile <script>alert(\"feature\")</script> & 'friends'",
        "@<b>feature-tag</b>",
        "@ticket:\"><script>alert('ticket')</script>",
        "Ticket \"><script>alert('ticket')</script>",
        "A description with <img src=x onerror=\"alert('description')\"> & <b>bold</b> &amp; &#60;i&#62;",
        "Feature: Hostile <script>alert(\"feature\")</script> & 'friends', hostile.feature",
        "Background: Setup <iframe src=\"javascript:alert('background')\"></iframe>",
        "A background's <video src=x onerror=\"alert('background description')\"></video>",
        "Given a user named \"<script>alert('parameter')</script>\"",
        "Scenario: Scenario <img src=x onerror=alert('scenario')> & \"quoted\"",
        "@<i>scenario-tag</i>",
        "Line one of <a href=\"javascript:alert('scenario description')\">a link</a> Line two with <!-- a comment --> and ]]>",
        "When the text is <i>italic</i> & \"<b>bold</b>\" with 'apostrophes' and 3 <u>numbers</u>",
        "<th>head</th>",
        "name & 'more'",
        "<script>alert('cell')</script>",
        "a | b <br> \"c\"",
        "<i>media</i> <script>alert('doc string')</script> </pre><b>outside the pre</b>",
        "Rule: Rule <svg onload=\"alert('rule')\"></svg>",
        "<a href=\"javascript:alert('rule description')\">A rule's link</a>",
        "Background: Rule's <style>body { display: none }</style>",
        "Given the style is \"<style>body { display: none }</style>\"",
        "Scenario outline: Outline <object data=\"x\"></object> for <who>",
        "@<em>outline-tag</em>",
        "An outline's <embed src=\"javascript:alert('outline description')\">",
        "These examples' <details open ontoggle=\"alert('examples description')\">",
        "Given \"<who>\" signs in as <who>",
        "Examples: Examples <textarea>written</textarea>",
        "@<s>examples-tag</s>",
        "Given \"<script>alert('row')</script>\" signs in as <script>alert('row')</script>",
        "Given \"<img src=x onerror=\"alert('row')\">\" signs in as <img src=x onerror=\"alert('row')\">",
    ]
)]
#[case::a_feature_in_a_folder_with_an_ampersand_and_an_apostrophe(
    "Q&A's notes/answers.html",
    &[
        "Feature: Answers & <b>questions</b> </title><script>alert('title')</script>, Q&A's notes/answers.feature",
        "Q&A's notes",
        "Feature: Answers & <b>questions</b> </title><script>alert('title')</script>",
        "Scenario: <marquee>Moving</marquee> & 'still'",
        "Given nothing <blink>blinks</blink>",
    ]
)]
#[case::the_folder_page(
    "Q&A's notes/index.html",
    &["Q&A's notes", "Feature: Answers & <b>questions</b> </title><script>alert('title')</script>"]
)]
#[case::the_home_page(
    "index.html",
    &[
        "Feature: Hostile <script>alert(\"feature\")</script> & 'friends'",
        "Q&A's notes",
    ]
)]
#[case::a_tag_page(
    "tags/ticket-script-alert-ticket-script.html",
    &[
        "@ticket:\"><script>alert('ticket')</script>",
        "Feature: Hostile <script>alert(\"feature\")</script> & 'friends'",
    ]
)]
#[case::the_tags_index("tags/index.html", &["@<b>feature-tag</b>", "@<s>examples-tag</s>"])]
#[case::the_step_sentences(
    "steps.html",
    &[
        "Background: Setup <iframe src=\"javascript:alert('background')\"></iframe>, hostile.feature, with \"<script>alert('parameter')</script>\"",
        "When the text is <i>italic</i> & \"…\" with 'apostrophes' and 3 <u>numbers</u>",
        "with \"<b>bold</b>\"",
        "Scenario: <marquee>Moving</marquee> & 'still', Q&A's notes/answers.feature",
        "nothing <blink>blinks</blink>",
    ]
)]
fn the_text_reads_as_written(#[case] page: &str, #[case] texts: &[&str]) {
    let text = page_text(&parsed(page));
    for written in texts {
        assert!(
            text.contains(written),
            "{page} does not read {written:?}:\n{text}"
        );
    }
}
