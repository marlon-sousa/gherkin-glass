//! The options of rules 232 and 233, read from the command line.

use rstest::rstest;

use super::*;
use crate::entities::Colour;
use crate::test_support::tag_link;

// Rule 232.
#[rstest]
#[case::a_source_link(&["build", "--source-link", "https://example.com/{path}#L{line}"], BuildOptions {
    source_link: Some("https://example.com/{path}#L{line}".into()),
    ..BuildOptions::default()
})]
#[case::false_and_true(
    &["build", "--allow-unnamed", "false", "--collapse-rules=true", "--collapse-scenarios", "true"],
    BuildOptions {
        allow_unnamed: Some(false),
        collapse_rules: Some(true),
        collapse_scenarios: Some(true),
        ..BuildOptions::default()
    }
)]
#[case::the_other_flags(
    &["build", "--show-tree", "false", "--inline-backgrounds=false", "--replace-existing-version", "false"],
    BuildOptions {
        show_tree: Some(false),
        inline_backgrounds: Some(false),
        replace_existing_version: Some(false),
        ..BuildOptions::default()
    }
)]
#[case::both_accents(&["build", "--accent-light", "#5A1E8C", "--accent-dark=#fc8"], BuildOptions {
    accent_light: Some(Colour::new(0x5a, 0x1e, 0x8c)),
    accent_dark: Some(Colour::new(0xff, 0xcc, 0x88)),
    ..BuildOptions::default()
})]
fn build_takes_an_option_for_each_key(#[case] arguments: &[&str], #[case] options: BuildOptions) {
    assert_eq!(parse(arguments), build(options));
}

// Rule 233: each link of three values, in the order given.
#[test]
fn build_takes_each_tag_link_with_its_three_values() {
    let arguments = [
        "build",
        "--tag-link",
        "@issue-(\\d+)",
        "https://example.com/issues/$1",
        "Issue $1",
        "--title",
        "Suite",
        "--tag-link",
        "@-(.+)",
        "-$1",
        "-",
    ];
    assert_eq!(
        parse(&arguments),
        build(BuildOptions {
            title: Some("Suite".into()),
            tag_links: Some(vec![
                tag_link("@issue-(\\d+)", "https://example.com/issues/$1", "Issue $1"),
                tag_link("@-(.+)", "-$1", "-"),
            ]),
            ..BuildOptions::default()
        })
    );
}

#[test]
fn serve_takes_the_options_of_the_keys_too() {
    let arguments = ["serve", "--source-link", "x", "--show-tree", "false"];
    let Ok(Invocation::Serve(serve)) = parse(&arguments) else {
        panic!("{arguments:?} is not a serve");
    };
    assert_eq!(serve.build.source_link.as_deref(), Some("x"));
    assert_eq!(serve.build.show_tree, Some(false));
}
