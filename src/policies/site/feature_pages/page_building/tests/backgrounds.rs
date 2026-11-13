use rstest::rstest;

use super::*;

#[test]
fn a_feature_s_and_a_rule_s_background_are_headed_each_in_its_own_words() {
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
    assert_eq!(background.block.heading_in_scenario, "From the background");
    assert_eq!(
        in_rule.block.heading_in_scenario,
        "From the rule's background"
    );
}

#[test]
fn an_unchecked_box_is_written_so() {
    let unchecked = PageOptions {
        inline_backgrounds: false,
        ..PageOptions::default()
    };
    assert_eq!(
        page(FEATURE, &unchecked)
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

/// Rule 51: only a background whose steps a scenario starts with starts in
/// the state the checkbox gives it; it is collapsed while the box is checked.
#[rstest]
#[case::with_its_scenarios("Feature: F\n  Background:\n    Given x\n  Scenario: S\n", Some(false))]
#[case::with_scenarios_only_in_a_rule(
    "Feature: F\n  Background:\n    Given x\n  Rule: R\n    Scenario: S\n",
    Some(false)
)]
#[case::without_steps("Feature: F\n  Background:\n  Scenario: S\n    Given x\n", None)]
#[case::without_scenarios("Feature: F\n  Background:\n    Given x\n  Rule: R\n", None)]
fn a_background_starts_collapsed_while_the_box_is_checked_when_a_scenario_holds_its_steps(
    #[case] text: &str,
    #[case] collapsed_when_unchecked: Option<bool>,
) {
    let background = |options: &PageOptions| {
        let page = page(text, options);
        let block = page.background.expect("a background").block;
        (block.heading.collapsed, block.collapsed_when_unchecked)
    };
    let held_inside = collapsed_when_unchecked.is_some();
    assert_eq!(
        background(&PageOptions::default()),
        (held_inside, collapsed_when_unchecked)
    );
    let unchecked = PageOptions {
        inline_backgrounds: false,
        ..PageOptions::default()
    };
    assert_eq!(background(&unchecked), (false, collapsed_when_unchecked));
    let scenarios = PageOptions {
        collapse_scenarios: true,
        ..unchecked
    };
    assert_eq!(
        background(&scenarios),
        (true, collapsed_when_unchecked.map(|_| true))
    );
}

/// Rule 51, for a rule's background, which only its rule's scenarios start with.
#[rstest]
#[case::with_its_scenarios(
    "Feature: F\n  Rule: R\n    Background:\n      Given x\n    Scenario: S\n",
    Some(false)
)]
#[case::with_scenarios_only_outside_it(
    "Feature: F\n  Scenario: S\n    Given y\n\n  Rule: R\n    Background:\n      Given x\n",
    None
)]
fn a_rule_s_background_follows_the_box_when_its_rule_has_scenarios(
    #[case] text: &str,
    #[case] collapsed_when_unchecked: Option<bool>,
) {
    let page = page(text, &PageOptions::default());
    let rule = page.rules.first().expect("a rule");
    let block = &rule.background.as_ref().expect("a background").block;
    assert_eq!(block.collapsed_when_unchecked, collapsed_when_unchecked);
    assert_eq!(block.heading.collapsed, collapsed_when_unchecked.is_some());
}
