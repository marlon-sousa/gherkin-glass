//! The options of rules 232 and 233, each over the key of the same meaning.

use super::*;
use crate::entities::{Colour, TagLink};
use crate::test_support::tag_link;

fn file_link() -> TagLink {
    tag_link("@file-(\\d+)", "https://example.com/$1", "File $1")
}

fn option_link() -> TagLink {
    tag_link("@option-(\\d+)", "https://example.com/$1", "Option $1")
}

/// A file that sets every key the options of rules 232 and 233 have, each away from its
/// default.
fn file() -> Config {
    Config {
        source_link: Some("https://example.com/file/{path}".into()),
        allow_unnamed: Some(false),
        collapse_rules: Some(true),
        collapse_scenarios: Some(true),
        show_tree: Some(false),
        inline_backgrounds: Some(false),
        replace_existing_version: Some(false),
        accent_light: Some(Colour::new(0x5a, 0x1e, 0x8c)),
        accent_dark: Some(Colour::new(0xff, 0xcc, 0x88)),
        tag_links: Some(vec![file_link()]),
        ..Config::default()
    }
}

/// Every option of rules 232 and 233, each the other way from [`file`].
fn options() -> BuildOptions {
    BuildOptions {
        source_link: Some("https://example.com/option/{path}".into()),
        allow_unnamed: Some(true),
        collapse_rules: Some(false),
        collapse_scenarios: Some(false),
        show_tree: Some(true),
        inline_backgrounds: Some(true),
        replace_existing_version: Some(true),
        accent_light: Some(Accents::DEFAULT.light),
        accent_dark: Some(Accents::DEFAULT.dark),
        tag_links: Some(vec![option_link()]),
        ..BuildOptions::default()
    }
}

fn resolved(options: &BuildOptions, config: &Config) -> Settings {
    resolve_settings(options, Path::new(""), config, Destination::Folder)
}

// Rules 25 and 232: each option wins over its key, false over true as well as true over false.
#[test]
fn each_option_wins_over_its_key() {
    let settings = resolved(&options(), &file());
    let pages = &settings.pages;
    assert_eq!(
        pages.source_link.as_deref(),
        Some("https://example.com/option/{path}")
    );
    assert_eq!(settings.unnamed, Unnamed::Allowed);
    assert!(!pages.collapse_rules);
    assert!(!pages.collapse_scenarios);
    assert!(pages.show_tree);
    assert!(pages.inline_backgrounds);
    assert!(settings.replace);
    assert_eq!(pages.accents, Accents::DEFAULT);
    assert_eq!(pages.tag_links, vec![option_link()]);
}

#[test]
fn each_key_wins_over_the_default_when_its_option_is_not_given() {
    let settings = resolved(&BuildOptions::default(), &file());
    let pages = &settings.pages;
    assert_eq!(
        pages.source_link.as_deref(),
        Some("https://example.com/file/{path}")
    );
    assert_eq!(settings.unnamed, Unnamed::Refused);
    assert!(pages.collapse_rules);
    assert!(pages.collapse_scenarios);
    assert!(!pages.show_tree);
    assert!(!pages.inline_backgrounds);
    assert!(!settings.replace);
    assert_eq!(pages.accents.light, Colour::new(0x5a, 0x1e, 0x8c));
    assert_eq!(pages.accents.dark, Colour::new(0xff, 0xcc, 0x88));
    assert_eq!(pages.tag_links, vec![file_link()]);
}

// Rule 233: the links given replace the file's, rather than adding to them.
#[test]
fn the_tag_links_given_replace_the_file_s_whole_list() {
    let file = Config {
        tag_links: Some(vec![file_link(), option_link()]),
        ..Config::default()
    };
    let options = BuildOptions {
        tag_links: Some(vec![option_link()]),
        ..BuildOptions::default()
    };
    assert_eq!(
        resolved(&options, &file).pages.tag_links,
        vec![option_link()]
    );
}

#[test]
fn an_option_that_sets_one_accent_leaves_the_other_to_the_file() {
    let options = BuildOptions {
        accent_light: Some(Accents::DEFAULT.light),
        ..BuildOptions::default()
    };
    let accents = resolved(&options, &file()).pages.accents;
    assert_eq!(accents.light, Accents::DEFAULT.light);
    assert_eq!(accents.dark, Colour::new(0xff, 0xcc, 0x88));
}
