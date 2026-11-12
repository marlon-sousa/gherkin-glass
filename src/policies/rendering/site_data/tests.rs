mod against_fixtures;

use super::*;
use crate::entities::LinkAddress;

#[test]
fn only_a_feature_s_item_has_a_lang_and_empty_lists_are_left_out_of_the_tree_data() {
    let item = |text: &str, address: &str, lang: Option<&str>, items| TreeItem {
        text: text.into(),
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
