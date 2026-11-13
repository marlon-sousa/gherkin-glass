use std::path::PathBuf;

use rstest::rstest;

use super::*;

fn read(text: &str) -> Result<MarkdownFrontPage, FrontPageError> {
    MarkdownFrontPage::of(&FrontPageTemplate {
        file: PathBuf::from("front.md"),
        text: text.to_string(),
    })
}

fn front_page(text: &str) -> MarkdownFrontPage {
    match read(text) {
        Ok(front_page) => front_page,
        Err(error) => panic!("{error}"),
    }
}

/// The line and the kind of what stops `text`.
fn refusal(text: &str) -> (Option<usize>, FrontPageErrorKind) {
    match read(text) {
        Ok(front_page) => panic!("{front_page:?}"),
        Err(error) => {
            assert_eq!(error.file, PathBuf::from("front.md"));
            (error.line, error.kind)
        }
    }
}

#[test]
fn the_marker_is_where_ghg_s_part_goes_between_the_template_s_html() {
    // Rules 185 and 190: the Markdown becomes HTML around ghg's part, tables included.
    let page = front_page(
        "# Ban*king* & `<more>`\n\nWhat it *is*.\n\n<!-- ghg:contents -->\n\n| a | b |\n| - | - |\n| 1 | 2 |\n",
    );
    assert_eq!(page.title, "Banking & <more>");
    assert_eq!(page.level, HeadingLevel::Two);
    assert_eq!(
        page.around("\n<ul><li>ghg</li></ul>"),
        "<h1>Ban<em>king</em> &amp; <code>&lt;more&gt;</code></h1>\n<p>What it <em>is</em>.</p>\n<ul><li>ghg</li></ul>\n<table><thead><tr><th>a</th><th>b</th></tr></thead><tbody>\n<tr><td>1</td><td>2</td></tr>\n</tbody></table>"
    );
}

#[test]
fn a_marker_first_and_nothing_after_it_leave_no_blank_line() {
    let page = front_page("<!-- ghg:contents -->\n# Title\n");
    assert_eq!(page.around("\n<p>ghg</p>"), "<p>ghg</p>\n<h1>Title</h1>");
    assert_eq!(page.level, HeadingLevel::Two);
}

#[test]
fn other_comments_are_left_out_and_links_kept_as_written() {
    // Rule 190.
    let page = front_page(
        "# T\n\n<!-- a note -->\n\nSee [the licence](LICENSE-MIT) <!-- inline -->.\n\n<!-- ghg:contents -->\n",
    );
    assert_eq!(
        page.around(""),
        "<h1>T</h1>\n<p>See <a href=\"LICENSE-MIT\">the licence</a> .</p>"
    );
}

#[rstest]
// Rule 189: ghg's headings go one level below the last heading before the marker.
#[case::under_the_title("# T\n<!-- ghg:contents -->\n", HeadingLevel::Two)]
#[case::under_a_level_two("# T\n## Versions\n<!-- ghg:contents -->\n", HeadingLevel::Three)]
#[case::under_a_level_four("# T\n## A\n#### B\n<!-- ghg:contents -->\n## C\n", HeadingLevel::Five)]
#[case::the_last_before_the_marker_only(
    "# T\n### A\n## B\n<!-- ghg:contents -->\n#### C\n",
    HeadingLevel::Three
)]
#[case::no_heading_before_it("<!-- ghg:contents -->\n# T\n", HeadingLevel::Two)]
fn ghg_s_headings_go_below_the_last_heading_before_the_marker(
    #[case] text: &str,
    #[case] level: HeadingLevel,
) {
    assert_eq!(front_page(text).level, level);
}

#[rstest]
// Rules 187 to 190: what stops a template, with its line when it has one.
#[case::no_title("Text\n\n<!-- ghg:contents -->\n", None, FrontPageErrorKind::NoTitle)]
#[case::a_second_title(
    "# A\n\n<!-- ghg:contents -->\n\nB\n=\n",
    Some(5),
    FrontPageErrorKind::SecondTitle
)]
#[case::no_marker("# A\n\n<!-- ghg:other -->\n", None, FrontPageErrorKind::NoMarker)]
#[case::a_second_marker(
    "# A\n<!-- ghg:contents -->\n\n<!-- ghg:contents -->\n",
    Some(4),
    FrontPageErrorKind::SecondMarker
)]
#[case::a_marker_in_a_list(
    "# A\n\n- <!-- ghg:contents -->\n",
    Some(3),
    FrontPageErrorKind::MarkerNotAlone
)]
#[case::a_marker_in_a_quote(
    "# A\n\n> <!-- ghg:contents -->\n",
    Some(3),
    FrontPageErrorKind::MarkerNotAlone
)]
#[case::a_marker_in_a_line_of_text(
    "# A\n\nSee <!-- ghg:contents --> here.\n",
    Some(3),
    FrontPageErrorKind::MarkerNotAlone
)]
#[case::a_marker_under_a_level_five(
    "# A\n##### B\n<!-- ghg:contents -->\n",
    Some(3),
    FrontPageErrorKind::MarkerTooDeep
)]
#[case::a_marker_under_a_level_six(
    "# A\n###### B\n<!-- ghg:contents -->\n",
    Some(3),
    FrontPageErrorKind::MarkerTooDeep
)]
#[case::an_html_block(
    "# A\n<!-- ghg:contents -->\n\n<div>\nx\n</div>\n",
    Some(4),
    FrontPageErrorKind::RawHtml
)]
#[case::inline_html(
    "# A\n<!-- ghg:contents -->\n\nSome <b>bold</b>.\n",
    Some(4),
    FrontPageErrorKind::RawHtml
)]
#[case::html_after_a_comment(
    "# A\n<!-- ghg:contents -->\n\n<!-- a --> <br>\n",
    Some(4),
    FrontPageErrorKind::RawHtml
)]
fn a_template_that_breaks_a_rule_stops_the_build(
    #[case] text: &str,
    #[case] line: Option<usize>,
    #[case] kind: FrontPageErrorKind,
) {
    assert_eq!(refusal(text), (line, kind));
}
