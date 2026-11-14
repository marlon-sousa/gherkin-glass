//! Headings shown away from their feature's page, and an outline's examples,
//! checked in the committed sites, which the tests of `tests/pages.rs`,
//! `tests/step_sentences.rs` and `tests/project_site.rs` compare with what
//! `ghg build` writes, byte for byte.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs::read_to_string;
use std::path::Path;

use rstest::rstest;

/// The committed page at `page`, below the repository's root.
fn committed(page: &str) -> String {
    read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(page)).expect("committed")
}

#[rstest]
#[case::a_folder_page_s_feature_link(
    "tests/fixtures/pages/expected/index.html",
    "<a href=\"saque%20em%20conta%20poupan%C3%A7a.html\">Feature: <bdi lang=\"pt\">Saque em conta poupança</bdi></a>"
)]
#[case::a_folder_page_s_contents(
    "tests/fixtures/pages/expected/index.html",
    "<div class=\"ghg-content\" hidden>\n<ul>\n<li><a href=\"saque%20em%20conta%20poupan%C3%A7a.html#saque-com-saldo\">Scenario: <bdi lang=\"pt\">Saque com saldo</bdi></a></li>"
)]
#[case::a_tag_page_s_feature(
    "project-site/expected/tags/empréstimo.html",
    "<h2><a href=\"../em-portugues/emprestimo.html\">Feature: <bdi lang=\"pt\">Empréstimo de livros</bdi>, "
)]
#[case::a_tag_page_s_scenario(
    "project-site/expected/tags/empréstimo.html",
    "<a href=\"../em-portugues/emprestimo.html#empréstimo-de-um-livro-na-estante\"><bdi>Scenario: <bdi lang=\"pt\">Empréstimo de um livro na estante</bdi></bdi></a>"
)]
#[case::a_use_of_a_step_sentence(
    "tests/fixtures/pages/expected/steps.html",
    "Scenario: <bdi lang=\"pt\">Saque com saldo</bdi>"
)]
fn on_an_english_page_only_a_heading_s_feature_text_has_its_feature_s_lang(
    #[case] page: &str,
    #[case] shown: &str,
) {
    // Rules 80, 105 and 144: gherkin-glass's own words are read in English.
    let html = committed(page);
    assert!(html.contains(shown), "{html}");
    let link_with_lang = html
        .split("<a ")
        .skip(1)
        .find(|link| link[..link.find('>').expect("closed")].contains("lang="));
    assert_eq!(link_with_lang, None);
}

#[test]
fn an_unnamed_examples_block_is_numbered_by_its_place_among_its_outline_s() {
    // Rule 43; the anchor stays the one of rule 31.
    let html = committed("tests/fixtures/pages/expected/outlines.html");
    assert!(
        html.contains("<li id=\"examples\" class=\"ghg-block ghg-examples\"><button type=\"button\" class=\"ghg-toggle\" aria-expanded=\"false\">Examples 1</button>"),
        "{html}"
    );
}

#[test]
fn an_outline_s_runs_are_headed_as_runs_and_an_unnamed_one_s_named_by_its_kind() {
    // Rules 129 and 130.
    let html = committed("tests/fixtures/pages/expected/outlines.html");
    assert!(
        html.contains("aria-expanded=\"false\">Runs, one per row</button>"),
        "{html}"
    );
    assert!(
        html.contains("aria-expanded=\"true\">Scenario outline (<bdi>x</bdi>: <bdi>1</bdi>"),
        "{html}"
    );
}

#[test]
fn an_examples_block_without_a_row_says_it_has_nothing_to_run_and_an_outline_with_none_does_not_say_it_runs()
 {
    // Rule 45, the list of an outline's examples.
    let html = committed("tests/fixtures/pages/expected/outlines.html");
    assert!(
        html.contains(
            "aria-expanded=\"false\">Examples: <bdi>None yet</bdi> (nothing to run)</button>"
        ),
        "{html}"
    );
    assert!(
        html.contains("aria-expanded=\"false\">Examples 1 (nothing to run)</button>"),
        "{html}"
    );
    // A block with nothing to show has a button that opens nothing.
    assert!(
        html.contains(
            "<li id=\"examples-3\" class=\"ghg-examples\"><button type=\"button\" class=\"ghg-toggle\" aria-disabled=\"true\">Examples 2 (nothing to run)</button></li>"
        ),
        "{html}"
    );
    let (with_rows, without) = html
        .split_once("id=\"nothing-to-produce\"")
        .expect("the outline whose examples have no row");
    assert_eq!(
        with_rows
            .matches("Runs once for each row of these examples:")
            .count(),
        2,
        "{with_rows}"
    );
    assert!(!without.contains("ghg-runs"), "{without}");
}

#[rstest]
#[case::the_feature_under_its_h1(
    "tests/fixtures/pages/expected/saque em conta poupança.html",
    "<h1>Feature: <bdi>Saque em conta poupança</bdi></h1>\n<p class=\"ghg-keyword-note\" role=\"note\">Note: in the Gherkin feature file, this feature is <button type=\"button\" class=\"ghg-note-opener\" aria-haspopup=\"dialog\" aria-controls=\"ghg_note_dialog\">defined with the keyword &#34;<bdi>Funcionalidade</bdi>&#34;.</button></p>"
)]
#[case::a_scenario_at_the_top_of_its_content(
    "tests/fixtures/pages/expected/kinds.html",
    "Scenario: <bdi>Written as an example</bdi></button></h3>\n<div class=\"ghg-content\">\n<p class=\"ghg-keyword-note\" role=\"note\">Note: in the Gherkin feature file, this scenario is <button type=\"button\" class=\"ghg-note-opener\" aria-haspopup=\"dialog\" aria-controls=\"ghg_note_dialog\">defined with the keyword &#34;<bdi>Example</bdi>&#34;.</button></p>"
)]
#[case::an_examples_block_inside_what_its_button_shows(
    "tests/fixtures/pages/expected/kinds.html",
    "<div class=\"ghg-content\" hidden>\n<p class=\"ghg-keyword-note\" role=\"note\">Note: in the Gherkin feature file, these examples are <button type=\"button\" class=\"ghg-note-opener\" aria-haspopup=\"dialog\" aria-controls=\"ghg_note_dialog\">defined with the keyword &#34;<bdi>Scenarios</bdi>&#34;.</button></p>"
)]
fn a_heading_names_only_its_kind_and_its_content_names_another_keyword(
    #[case] page: &str,
    #[case] shown: &str,
) {
    // Rules 43 and 196.
    let html = committed(page);
    assert!(html.contains(shown), "{html}");
    assert!(!html.contains("written as"), "{html}");
}

#[rstest]
#[case::with_notes("tests/fixtures/pages/expected/kinds.html", true)]
#[case::without_one("tests/fixtures/pages/expected/outlines.html", false)]
fn a_page_with_a_keyword_s_note_has_one_closed_dialog_that_explains_it(
    #[case] page: &str,
    #[case] has_notes: bool,
) {
    // Rule 197.
    let html = committed(page);
    assert_eq!(
        html.contains("class=\"ghg-note-opener\""),
        has_notes,
        "{html}"
    );
    let dialogs = html.matches("<dialog id=\"ghg_note_dialog\"").count();
    assert_eq!(dialogs, usize::from(has_notes), "{html}");
    if has_notes {
        assert!(
            html.contains(
                "<dialog id=\"ghg_note_dialog\" class=\"ghg-dialog\" tabindex=\"-1\" aria-labelledby=\"ghg_note_dialog_title\" aria-describedby=\"ghg_note_dialog_text\">\n<h2 id=\"ghg_note_dialog_title\" class=\"ghg-dialog-title\">About this note</h2>\n<p id=\"ghg_note_dialog_text\">Gherkin lets authors use different words for the same kind of block. "
            ),
            "{html}"
        );
        assert!(
            html.contains("<form method=\"dialog\"><button>Got it</button></form>\n</dialog>"),
            "{html}"
        );
    }
}
