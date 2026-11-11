//! Doc strings, read as their author wrote them, checked against the fixture.

use super::*;

// Doc strings: rules 9 to 13.

/// Each doc string of the fixture, with its media type, in the order of the file.
fn doc_strings(input: &Path) -> Vec<(Option<String>, String)> {
    let features = read(input);
    let parsed = &features[0];
    let feature = &parsed.feature;
    let steps: Vec<&Step> = feature
        .background
        .iter()
        .flat_map(|background| &background.steps)
        .chain(
            feature
                .scenarios
                .iter()
                .flat_map(|scenario| &scenario.steps),
        )
        .chain(
            feature
                .rules
                .iter()
                .flat_map(|rule| rule.scenarios.iter().flat_map(|scenario| &scenario.steps)),
        )
        .collect();
    steps
        .into_iter()
        .filter_map(|step| {
            let content = step.docstring.clone()?;
            Some((parsed.media_type(step).map(str::to_string), content))
        })
        .collect()
}

fn expected_doc_strings() -> Vec<(Option<String>, String)> {
    [
        (None, "From the background."),
        // Rules 9, 11 and 12.
        (Some("json"), "{\n  \"quote\": \"\"\"\"\"\n}"),
        // Rules 9 and 12.
        (Some("markdown"), "Code fences look like ``` in Markdown."),
        // Rule 11.
        (
            None,
            "indented as the delimiter\nindented less\n    indented more",
        ),
        // Rule 10: blank lines inside are kept, and a comment before is not content.
        (None, "\nbetween blank lines\n"),
        // Rule 11: a tab is one character of indentation. Rule 10: a line
        // that looks like a comment is content, and a delimiter on it does
        // not close the doc string.
        (
            None,
            "one tab, as indentation\n\ta tab after the indentation\n# A line like a comment, with \"\"\" in it, is content.\nx = 1 # and from a hash on any line, \"\"\" does not close it either",
        ),
        (Some("text"), "Inside the rule."),
    ]
    .into_iter()
    .map(|(media_type, content)| (media_type.map(str::to_string), content.to_string()))
    .collect()
}

#[test]
fn doc_strings_are_read_as_the_author_wrote_them() {
    assert_eq!(doc_strings(&fixture("doc-strings")), expected_doc_strings());
}

#[test]
fn doc_strings_read_the_same_with_carriage_returns() {
    // Rule 13.
    let input = scratch("input/doc-strings-crlf");
    let source = read_to_string(fixture("doc-strings/doc-strings.feature")).expect("read");
    let crlf = source.replace("\r\n", "\n").replace('\n', "\r\n");
    write(input.join("doc-strings.feature"), crlf).expect("written");

    assert_eq!(doc_strings(&input), expected_doc_strings());
}
