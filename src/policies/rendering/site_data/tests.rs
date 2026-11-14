mod against_fixtures;

use super::*;
use crate::entities::{LinkAddress, plain_parts};

#[test]
fn only_a_feature_s_item_has_a_lang_and_empty_lists_are_left_out_of_the_tree_data() {
    let item = |text: &str, address: &str, lang: Option<&str>, items| TreeItem {
        text: plain_parts(text),
        address: LinkAddress::to(address),
        lang: lang.map(str::to_string),
        items,
    };
    let scenario = TreeItem {
        address: LinkAddress::to("a/b.html").at("s"),
        ..item("Scenario: S", "", None, Vec::new())
    };
    let feature = item("Feature: B", "a/b.html", Some("en"), vec![scenario]);
    let items = [
        item("Site", "index.html", None, Vec::new()),
        item("a", "a/index.html", None, vec![feature]),
    ];
    assert_eq!(
        items.render(),
        "[{\"text\":\"Site\",\"address\":\"index.html\"},\
         {\"text\":\"a\",\"address\":\"a/index.html\",\"items\":[\
         {\"text\":\"Feature: B\",\"address\":\"a/b.html\",\"lang\":\"en\",\"items\":[\
         {\"text\":\"Scenario: S\",\"address\":\"a/b.html#s\"}]}]}]\n"
    );
}

#[test]
fn an_item_with_isolated_words_holds_its_runs_the_isolated_at_odd_places() {
    let part = |text: &str, isolated| HeadingPart {
        text: text.to_string(),
        isolated,
    };
    let item = TreeItem {
        text: vec![
            part("Scenario", false),
            part(": ", false),
            part("S", true),
            part(" (written as ", false),
            part("Example", true),
        ],
        address: LinkAddress::to("b.html").at("s"),
        lang: None,
        items: Vec::new(),
    };
    assert_eq!(
        [item].render(),
        "[{\"text\":[\"Scenario: \",\"S\",\" (written as \",\"Example\"],\"address\":\"b.html#s\"}]\n"
    );
}
