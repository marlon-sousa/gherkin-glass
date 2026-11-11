//! docs/BEHAVIOUR.md, "Step sentences", checked against the fixtures: the
//! sentences found, and the page of step sentences compared with the one
//! committed beside each fixture.
//!
//! `ghg build --input . --title Fixture --output <folder>`, run in the
//! fixture's folder, writes the page again after a change, to be read before
//! it is committed.

use std::fs::{read_dir, read_to_string, remove_dir_all};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use gherkin_glass::{
    GherkinParser, InputService, InputWalker, StepKind, Unnamed, find_step_sentences, make_anchors,
};

/// A use of a sentence: its path, anchor, line and values.
type Use = (String, String, usize, Vec<String>);

/// Each group's kind, and each of its sentences with every use.
type Found = Vec<(StepKind, Vec<(String, Vec<Use>)>)>;

fn found() -> Found {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/step-sentences");
    let features = InputService::new(Arc::new(InputWalker), Arc::new(GherkinParser))
        .read(&input, Unnamed::Allowed)
        .unwrap_or_else(|problems| panic!("the input is read:\n{problems}"));
    let anchors: Vec<_> = features
        .iter()
        .map(|parsed| make_anchors(&parsed.feature))
        .collect();
    find_step_sentences(features.iter().zip(&anchors))
        .groups
        .into_iter()
        .map(|group| {
            let sentences = group
                .sentences
                .into_iter()
                .map(|sentence| {
                    let uses = sentence
                        .uses
                        .into_iter()
                        .map(|found| {
                            let path = found.path.to_string_lossy().replace('\\', "/");
                            (path, found.anchor, found.line, found.values)
                        })
                        .collect();
                    (sentence.text, uses)
                })
                .collect();
            (group.kind, sentences)
        })
        .collect()
}

fn sentence(text: &str, uses: &[(&str, &str, usize, &[&str])]) -> (String, Vec<Use>) {
    let uses = uses
        .iter()
        .map(|(path, anchor, line, values)| {
            let values = values.iter().map(|value| value.to_string()).collect();
            (path.to_string(), anchor.to_string(), *line, values)
        })
        .collect();
    (text.to_string(), uses)
}

const LOGGING_IN: &str = "accounts/logging-in.feature";
const OLD: &str = "accounts-old.feature";
const ESCAPES: &str = "escapes.feature";
const MONEY: &str = "money.feature";
const PORTUGUESE: &str = "em-portugues.feature";

#[test]
fn every_sentence_is_grouped_sorted_and_linked_to_its_uses() {
    let expected: Found = vec![
        (
            StepKind::Given,
            vec![
                // Rule 137: a * step is a Given step, and so is an And step after it.
                sentence("a star step", &[(OLD, "star-steps", 7, &[])]),
                sentence("a step after a star", &[(OLD, "star-steps", 8, &[])]),
                // Rule 133: the doc string and the data table are not part of the sentence.
                sentence("a step with a doc string:", &[(OLD, "star-steps", 11, &[])]),
                sentence("a step with a table:", &[(OLD, "star-steps", 9, &[])]),
                // Rule 133: Portuguese steps are Given steps, and compared as written.
                sentence(
                    "o usuário \"…\" entra",
                    &[(PORTUGUESE, "ana-entra", 5, &["\"ana\""])],
                ),
                // Rule 137: a first step written with And is a Given step.
                sentence(
                    "the first step is an and",
                    &[(MONEY, "starting-with-and", 12, &[])],
                ),
                // Rule 135: the columns of every examples block are placeholders.
                sentence(
                    "the user \"…\" and \"…\" meet",
                    &[(OLD, "two-examples", 17, &["<first>", "<second>"])],
                ),
                // Rule 140: a background's step links to the background, and the
                // uses come in the order of the paths, then of the lines.
                // Rule 135: a quoted placeholder is one slot.
                sentence(
                    "the user \"…\" exists",
                    &[
                        (LOGGING_IN, "background", 4, &["\"ana\""]),
                        (LOGGING_IN, "anyone-logs-in", 18, &["\"<name>\""]),
                        // Rule 140: folder by folder, so accounts/ comes before accounts-old.
                        (OLD, "star-steps", 4, &["\"fay\""]),
                        // Rule 133: a step in Portuguese, written as an English one.
                        (PORTUGUESE, "gil", 12, &["\"gil\""]),
                        (MONEY, "background", 17, &["\"dan\""]),
                    ],
                ),
                // Rules 134 and 135: quoted values and an outline's placeholder at
                // the same place are one sentence; rule 137: a * step is a Given step.
                sentence(
                    "the user \"…\" logs in",
                    &[
                        (LOGGING_IN, "ana-logs-in", 7, &["\"ana\""]),
                        (LOGGING_IN, "bia-logs-in", 13, &["\"bia\""]),
                        (LOGGING_IN, "anyone-logs-in", 17, &["<name>"]),
                    ],
                ),
            ],
        ),
        (
            StepKind::When,
            vec![
                sentence("ela abre a conta", &[(PORTUGUESE, "ana-entra", 6, &[])]),
                // Rule 135: a placeholder is a slot, shown as a quoted value.
                sentence(
                    "I withdraw \"…\"",
                    &[(LOGGING_IN, "anyone-logs-in", 19, &["<amount>"])],
                ),
                // Rule 136: numbers are text.
                sentence("I withdraw 10", &[(MONEY, "withdraw", 4, &[])]),
                sentence("I withdraw 20", &[(MONEY, "withdraw", 5, &[])]),
                sentence(
                    "she opens her account",
                    &[(LOGGING_IN, "ana-logs-in", 8, &[])],
                ),
                sentence("the old account is opened", &[(OLD, "star-steps", 5, &[])]),
                // Rule 137: the same text in two kinds is two sentences.
                sentence(
                    "the user \"…\" logs in",
                    &[(MONEY, "eve-logs-in", 20, &["\"eve\""])],
                ),
            ],
        ),
        (
            StepKind::Then,
            vec![
                // Rule 137: a But step, here Mas, takes the kind before it.
                sentence(
                    "ela não vê \"…\"",
                    &[(PORTUGUESE, "ana-entra", 9, &["\"Erro\""])],
                ),
                // Rule 139: two uses in one scenario count twice.
                sentence(
                    "ela vê \"…\"",
                    &[
                        (PORTUGUESE, "ana-entra", 7, &["\"Bem-vinda\""]),
                        (PORTUGUESE, "ana-entra", 8, &["\"Bem-vinda\""]),
                    ],
                ),
                sentence("it is shown", &[(OLD, "star-steps", 6, &[])]),
                // Rule 138: sorted lower-cased, capitals first between equals.
                sentence(
                    "She sees \"…\"",
                    &[(LOGGING_IN, "bia-logs-in", 14, &["\"Welcome\""])],
                ),
                sentence(
                    "she sees \"…\"",
                    &[
                        (LOGGING_IN, "ana-logs-in", 9, &["\"Welcome\""]),
                        (LOGGING_IN, "ana-logs-in", 10, &["\"Welcome\""]),
                    ],
                ),
                // Rule 135: < and > that do not name a column are text.
                sentence(
                    "the balance is < 0 and <unknown>",
                    &[(LOGGING_IN, "anyone-logs-in", 20, &[])],
                ),
                // Rule 133: compared in NFC.
                sentence(
                    "the café is paid",
                    &[
                        (MONEY, "withdraw", 9, &[]),
                        (MONEY, "starting-with-and", 13, &[]),
                    ],
                ),
                // Rule 134: an empty value is a slot.
                sentence(
                    "the note says \"…\" and \"…\"",
                    &[(MONEY, "withdraw", 6, &["\"\"", "\"it's done\""])],
                ),
                // Rule 135: an empty name is no placeholder, even of an empty column.
                sentence(
                    "the screen shows <> for \"…\"",
                    &[(ESCAPES, "quotes-after-a-backslash", 5, &["<name>"])],
                ),
                // Rule 134: a quote just after a backslash neither opens nor closes a slot.
                sentence(
                    r#"the screen shows \"hi\" and "…" for "…""#,
                    &[(
                        ESCAPES,
                        "quotes-after-a-backslash",
                        4,
                        &[r#""a \" b""#, "<name>"],
                    )],
                ),
                // Rule 134: other quotation marks are text.
                sentence("the screen shows “hi”", &[(MONEY, "withdraw", 8, &[])]),
                // Rule 135: outside an outline, a placeholder is text.
                sentence(
                    "the user <name> logs in",
                    &[(MONEY, "eve-logs-in", 21, &[])],
                ),
                // Rule 134: a quote with no other after it is text.
                sentence("the user's card is \"kept", &[(MONEY, "withdraw", 7, &[])]),
            ],
        ),
    ];
    assert_eq!(found(), expected);
}

/// The page of step sentences `ghg build` writes for the fixture `name`,
/// and the one committed beside it.
fn pages(name: &str) -> (String, String) {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let written = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("sentences-{name}"));
    let _ = remove_dir_all(&written);
    let result = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(["build", "--input", ".", "--title", "Fixture", "--output"])
        .arg(&written)
        .current_dir(&fixture)
        .output()
        .expect("the command runs");
    assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    assert_eq!(result.status.code(), Some(0));
    (
        read_to_string(written.join("steps.html")).expect("the page is written"),
        read_to_string(fixture.join("expected/steps.html")).expect("the page is committed"),
    )
}

#[test]
fn the_page_shows_every_sentence_under_its_heading_with_links_to_its_uses() {
    // Rules 142 to 144.
    let (written, expected) = pages("step-sentences");
    assert!(written == expected, "steps.html differs:\n{written}");
}

#[test]
fn a_suite_with_no_step_has_a_page_that_says_so() {
    // Rule 145.
    let (written, expected) = pages("no-steps");
    assert!(written == expected, "steps.html differs:\n{written}");
}

#[test]
fn a_background_s_steps_are_listed_once_whether_inline_or_linked() {
    // Rule 140: the backgrounds fixture's two sites, written inline and linked.
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backgrounds");
    let inline = read_to_string(fixture.join("expected/steps.html")).expect("committed");
    let linked = read_to_string(fixture.join("expected-linked/steps.html")).expect("committed");
    assert!(inline == linked, "the two forms list the same sentences");
    let background = "<h3>a limit of 50</h3>\n<p>Used once.</p>\n<ul>\n\
                      <li><a href=\"both.html#over-the-limit\">";
    assert!(inline.contains(background), "{inline}");
}

#[test]
fn only_the_home_page_and_the_page_itself_link_the_page_of_step_sentences() {
    // Rule 146, in the pages fixture's site.
    let site = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pages/expected");
    let home = read_to_string(site.join("index.html")).expect("committed");
    assert!(
        home.contains(
            "</ul>\n<p class=\"ghg-tag-index-link\"><a href=\"tags/index.html\">Tags</a></p>\n\
             <p class=\"ghg-sentences\"><a href=\"steps.html\">Step sentences</a></p>\n<h2>"
        ),
        "{home}"
    );
    let mut linking = Vec::new();
    let mut folders = vec![site.clone()];
    while let Some(folder) = folders.pop() {
        for entry in read_dir(&folder).expect("listed") {
            let path = entry.expect("listed").path();
            if path.is_dir() {
                folders.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "html")
                && read_to_string(&path)
                    .expect("read")
                    .contains("steps.html\"")
            {
                let name = path.strip_prefix(&site).expect("inside");
                linking.push(name.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    linking.sort();
    assert_eq!(linking, ["index.html", "steps.html"]);
}
