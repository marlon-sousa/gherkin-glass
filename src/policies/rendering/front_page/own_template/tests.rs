use proptest::prelude::proptest;
use rstest::rstest;

use super::*;
use crate::entities::HeadingLevel;
use crate::policies::rendering::front_page::markdown_template::MarkdownFrontPage;

#[rstest]
#[case::plain("Feature files", "Feature files")]
#[case::html("Banking & <more>", "Banking &amp; &lt;more&gt;")]
#[case::emphasis("*a* _b_ `c`", "*a* _b_ `c`")]
#[case::a_heading_s_mark("# hash #", "# hash #")]
#[case::a_list_s_mark("1. one", "1. one")]
#[case::a_rule("---", "---")]
#[case::the_marker("<!-- ghg:contents -->", "&lt;!-- ghg:contents --&gt;")]
#[case::a_backslash("a \\ b", "a \\ b")]
#[case::a_link("[x](y)", "[x](y)")]
#[case::accents("Été — ok", "Été — ok")]
fn the_site_s_title_is_read_back_as_plain_text(#[case] title: &str, #[case] h1: &str) {
    // Rule 184: no title is read as Markdown or stops a build.
    let page =
        MarkdownFrontPage::of_own(&own_template(title)).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(page.title, title);
    assert_eq!(page.level, HeadingLevel::Two);
    assert_eq!(
        page.around("\n<p>ghg</p>"),
        format!("<h1>{h1}</h1>\n<p>ghg</p>")
    );
}

#[rstest]
#[case::spaces("   ")]
#[case::a_vertical_tab("\u{b}")]
fn a_title_markdown_reads_as_blank_still_builds(#[case] title: &str) {
    // Rule 184: no title stops a build.
    MarkdownFrontPage::of_own(&own_template(title)).unwrap_or_else(|error| panic!("{error}"));
}

proptest! {
    // Rule 184: any title without spaces or line breaks at its ends, of any printable
    // characters, is read back as it was given.
    #[test]
    fn any_title_is_read_back_as_given(title in "[^\\s\\p{C}]([^\\p{C}]{0,30}[^\\s\\p{C}])?") {
        let page = MarkdownFrontPage::of_own(&own_template(&title))
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(page.title, title);
    }

    // Rule 184: no title stops a build. A title has at least one character, since an empty
    // one is refused, but it may be of any kind, control characters included.
    #[test]
    fn no_title_stops_a_build(title in "(?s).{1,30}|[ \\t\\r\\n]{1,5}") {
        MarkdownFrontPage::of_own(&own_template(&title)).unwrap_or_else(|error| panic!("{error}"));
    }
}
