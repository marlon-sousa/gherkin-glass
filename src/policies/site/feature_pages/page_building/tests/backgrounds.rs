use rstest::rstest;

use super::*;

#[test]
fn a_feature_s_and_a_rule_s_background_are_introduced_each_in_its_own_words() {
    let page = page(FEATURE, &PageOptions::default());
    assert_eq!(
        page.page.backgrounds_option,
        Some(BackgroundsOption {
            label: "Show backgrounds inside each scenario".into(),
            inline: true,
        })
    );
    let (background, _, rule) = blocks(&page);
    let (in_rule, _) = in_rule(rule);
    assert_eq!(background.block.introduction, "From the background:");
    assert_eq!(in_rule.block.introduction, "From the rule's background:");
}

#[test]
fn linked_backgrounds_are_written_so() {
    let linked = PageOptions {
        inline_backgrounds: false,
        ..PageOptions::default()
    };
    assert_eq!(
        page(FEATURE, &linked)
            .page
            .backgrounds_option
            .map(|o| o.inline),
        Some(false)
    );
}

#[rstest]
#[case::with_a_background("Feature: F\n  Background:\n    Only words.\n", true)]
#[case::with_one_in_a_rule("Feature: F\n  Rule: R\n    Background:\n      Given x\n", true)]
#[case::with_no_background("Feature: F\n  Scenario: S\n    Given x\n", false)]
fn the_option_is_offered_on_a_page_with_a_background(#[case] text: &str, #[case] offered: bool) {
    assert_eq!(
        page(text, &PageOptions::default())
            .page
            .backgrounds_option
            .is_some(),
        offered
    );
}
