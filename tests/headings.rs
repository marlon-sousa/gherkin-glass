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
    "<a href=\"saque%20em%20conta%20poupan%C3%A7a.html\">Feature: <bdi lang=\"pt\">Saque em conta poupança</bdi> (written as <bdi lang=\"pt\">Funcionalidade</bdi>)</a>"
)]
#[case::a_folder_page_s_contents(
    "tests/fixtures/pages/expected/index.html",
    "<div class=\"ghg-content\" hidden>\n<ul>\n<li><a href=\"saque%20em%20conta%20poupan%C3%A7a.html#saque-com-saldo\">Scenario: <bdi lang=\"pt\">Saque com saldo</bdi> (written as <bdi lang=\"pt\">Cenário</bdi>)</a></li>"
)]
#[case::a_tag_page_s_feature(
    "project-site/expected/tags/empréstimo.html",
    "<h2><a href=\"../em-portugues/emprestimo.html\">Feature: <bdi lang=\"pt\">Empréstimo de livros</bdi> (written as <bdi lang=\"pt\">Funcionalidade</bdi>), "
)]
#[case::a_tag_page_s_scenario(
    "project-site/expected/tags/empréstimo.html",
    "<a href=\"../em-portugues/emprestimo.html#empréstimo-de-um-livro-na-estante\"><bdi>Scenario: <bdi lang=\"pt\">Empréstimo de um livro na estante</bdi> (written as <bdi lang=\"pt\">Cenário</bdi>)</bdi></a>"
)]
#[case::a_use_of_a_step_sentence(
    "tests/fixtures/pages/expected/steps.html",
    "Scenario: <bdi lang=\"pt\">Saque com saldo</bdi> (written as <bdi lang=\"pt\">Cenário</bdi>)"
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
fn an_unnamed_outline_is_named_by_its_kind_in_its_examples_as_scenarios() {
    // Rules 129 and 130.
    let html = committed("tests/fixtures/pages/expected/outlines.html");
    assert!(
        html.contains("aria-expanded=\"false\">Examples as scenarios: Scenario outline</button>"),
        "{html}"
    );
    assert!(
        html.contains("aria-expanded=\"true\">Scenario outline (<bdi>x</bdi>: <bdi>1</bdi>"),
        "{html}"
    );
}

#[test]
fn an_examples_block_without_a_row_says_so_and_an_outline_with_none_does_not_say_it_runs() {
    // Rule 45, the list of an outline's examples.
    let html = committed("tests/fixtures/pages/expected/outlines.html");
    assert!(
        html.contains("aria-expanded=\"false\">Examples: <bdi>None yet</bdi>: empty</button>"),
        "{html}"
    );
    assert!(
        html.contains("aria-expanded=\"false\">Examples: empty</button>"),
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
