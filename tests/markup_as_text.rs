//! Rule 203: markup in a feature file reaches every page as text, checked by
//! building `tests/fixtures/hostile`, parsing each file of HTML it writes as
//! a browser does, with `scraper`, and reading the JSON of the site's data.

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
use serde_json::Value;

/// Every element the shell and its front page hold.
const ELEMENTS: [&str; 29] = [
    "a", "bdi", "body", "button", "dialog", "div", "fieldset", "footer", "form", "h1", "h2",
    "head", "html", "input", "label", "legend", "li", "link", "main", "meta", "nav", "noscript",
    "ol", "p", "script", "span", "style", "title", "ul",
];

/// Every attribute the shell and its front page hold.
const ATTRIBUTES: [&str; 47] = [
    "aria-autocomplete",
    "aria-controls",
    "aria-describedby",
    "aria-expanded",
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
    "data-failed",
    "data-feature",
    "data-ghg-descriptor",
    "data-loading",
    "data-no-name",
    "data-none",
    "data-numbered-examples",
    "data-rule",
    "data-scenario",
    "data-scenario-in-rule",
    "data-scenario-outline",
    "data-scenario-outline-in-rule",
    "data-shown",
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
    "spellcheck",
    "src",
    "tabindex",
    "type",
    "value",
    "aria-busy",
];

/// The elements ghg writes only in a page's head.
const HEAD_ONLY: [&str; 4] = ["link", "meta", "style", "title"];

/// The scripts a shell runs: the theme's line, the line that opens the dialog
/// for a page opened from disk, and the pages' script.
const SCRIPTS_RUN: usize = 3;

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

/// Every file of the site, by its `/`-joined path in it, whose name ends
/// with `ending`.
fn files_ending_with(ending: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut folders = vec![(site().to_path_buf(), String::new())];
    while let Some((folder, prefix)) = folders.pop() {
        for entry in read_dir(&folder).expect("the folder is listed") {
            let entry = entry.expect("the folder is listed");
            let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
            if entry.path().is_dir() {
                folders.push((entry.path(), format!("{name}/")));
            } else if name.ends_with(ending) {
                found.push(name);
            }
        }
    }
    found.sort();
    found
}

fn pages() -> Vec<String> {
    files_ending_with(".html")
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

fn strings_in(value: &Value, found: &mut Vec<String>) {
    match value {
        Value::String(text) => found.push(text.clone()),
        Value::Array(values) => values.iter().for_each(|value| strings_in(value, found)),
        Value::Object(members) => members.values().for_each(|value| strings_in(value, found)),
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

/// Every string the site's data holds, in all its files.
fn data_strings() -> Vec<String> {
    let mut found = Vec::new();
    for file in files_ending_with(".json") {
        let text = read_to_string(site().join(&file)).expect("the file is read");
        let value: Value = serde_json::from_str(&text).expect("the file is JSON");
        strings_in(&value, &mut found);
    }
    found
}

#[test]
fn the_hostile_fixture_writes_its_shell_alone_as_html() {
    assert_eq!(pages(), ["index.html"]);
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

// Rule 203: the only scripts run are ghg's own; the page messages are text.
#[test]
fn every_page_runs_only_ghg_s_own_scripts() {
    let run = Selector::parse("script:not([type=\"text/plain\"])").expect("the selector is valid");
    let text = Selector::parse("script[type=\"text/plain\"]").expect("the selector is valid");
    for page in pages() {
        let html = parsed(&page);
        assert_eq!(html.select(&run).count(), SCRIPTS_RUN, "{page}");
        assert_eq!(html.select(&text).count(), 1, "{page}");
    }
}

// Rule 203: the HTML writes the text with character references, and the
// addresses made from it percent-encoded.
#[rstest]
#[case::the_feature_s_name(
    "Feature: <bdi lang=\"en\">Hostile &#60;script&#62;alert(&#34;feature&#34;)&#60;/script&#62; &#38; &#39;friends&#39;</bdi>"
)]
#[case::a_folder_with_an_ampersand_and_an_apostrophe(
    "<a href=\"#Q%26A%27s%20notes/\">Q&#38;A&#39;s notes</a>"
)]
fn the_front_page_writes_the_text_escaped(#[case] written: &str) {
    let html = read_to_string(site().join("index.html")).expect("the page is read");
    assert!(
        html.contains(written),
        "the front page does not hold {written}"
    );
}

// Rule 203: the front page reads the text as written in the feature files.
#[test]
fn the_front_page_reads_the_text_as_written() {
    let text = page_text(&parsed("index.html"));
    for written in [
        "Feature: Hostile <script>alert(\"feature\")</script> & 'friends'",
        "Q&A's notes",
        "Rule: Rule <svg onload=\"alert('rule')\"></svg>",
        "Scenario outline: Outline <object data=\"x\"></object> for <who>",
    ] {
        assert!(
            text.contains(written),
            "the front page does not read {written:?}:\n{text}"
        );
    }
}

// Rule 203: the site's data holds the text as written in the feature files,
// and the addresses made from it percent-encoded.
#[rstest]
#[case::a_feature_s_name("Hostile <script>alert(\"feature\")</script> & 'friends'")]
#[case::a_description(
    "A description with <img src=x onerror=\"alert('description')\"> & <b>bold</b> &amp; &#60;i&#62;"
)]
#[case::a_tag("@ticket:\"><script>alert('ticket')</script>")]
#[case::a_tag_s_link_text("Ticket \"><script>alert('ticket')</script>")]
#[case::a_tag_s_link(
    "https://example.com/tickets/%22%3E%3Cscript%3Ealert%28%27ticket%27%29%3C/script%3E"
)]
#[case::a_background_s_name("Setup <iframe src=\"javascript:alert('background')\"></iframe>")]
#[case::a_parameter("\"<script>alert('parameter')</script>\"")]
#[case::a_scenario_s_name("Scenario <img src=x onerror=alert('scenario')> & \"quoted\"")]
#[case::a_description_of_two_lines(
    "Line one of <a href=\"javascript:alert('scenario description')\">a link</a>\nLine two with <!-- a comment --> and ]]>"
)]
#[case::a_step(
    "the text is <i>italic</i> & \"<b>bold</b>\" with 'apostrophes' and 3 <u>numbers</u>"
)]
#[case::a_header_cell("<th>head</th>")]
#[case::a_cell("<script>alert('cell')</script>")]
#[case::a_cell_with_a_pipe("a | b <br> \"c\"")]
#[case::a_media_type("<i>media</i>")]
#[case::a_doc_string("<script>alert('doc string')</script>\n</pre><b>outside the pre</b>")]
#[case::a_rule_s_description("<a href=\"javascript:alert('rule description')\">A rule's link</a>")]
#[case::an_examples_block_s_name("Examples <textarea>written</textarea>")]
#[case::an_examples_tag("@<s>examples-tag</s>")]
#[case::a_row_s_step(
    "\"<img src=x onerror=\"alert('row')\">\" signs in as <img src=x onerror=\"alert('row')\">"
)]
#[case::a_feature_s_name_with_a_title(
    "Answers & <b>questions</b> </title><script>alert('title')</script>"
)]
#[case::a_file_in_a_folder_with_an_ampersand_and_an_apostrophe("Q&A's notes/answers.feature")]
#[case::its_source_link(
    "https://example.com/repo/blob/main/features/Q%26A%27s%20notes/answers.feature#L1"
)]
#[case::a_step_sentence("nothing <blink>blinks</blink>")]
fn the_data_holds_the_text_as_written(#[case] written: &str) {
    assert!(
        data_strings().iter().any(|text| text == written),
        "the site's data does not hold {written:?}"
    );
}
