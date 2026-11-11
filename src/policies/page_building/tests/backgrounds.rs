use super::*;

#[test]
fn each_scenario_starts_with_the_feature_s_background_then_its_rule_s() {
    let page = page(FEATURE, &PageOptions::default());
    fn from(block: &Block) -> Vec<(&str, &str, &str, &str, bool)> {
        block
            .backgrounds
            .iter()
            .map(|from| {
                (
                    from.introduction.as_str(),
                    from.anchor.as_str(),
                    from.heading.as_str(),
                    match from.steps[0].parts.as_slice() {
                        [StepPart::Plain(text)] => text.as_str(),
                        other => panic!("{other:?}"),
                    },
                    from.inline,
                )
            })
            .collect()
    }
    let feature = (
        "From the background:",
        "background",
        "Background",
        "a customer",
        true,
    );
    assert!(
        page.blocks[0].backgrounds.is_empty(),
        "the background's own"
    );
    assert_eq!(from(&page.blocks[1]), [feature]);
    let rule = &page.blocks[2];
    assert!(rule.backgrounds.is_empty());
    assert_eq!(
        from(&rule.blocks[1]),
        [
            feature,
            (
                "From the rule's background:",
                "in-the-rule",
                "Background: In the rule",
                "y",
                true
            )
        ]
    );
    assert!(rule.blocks[1].blocks[0].backgrounds.is_empty(), "examples");
    assert_eq!(
        page.backgrounds_option,
        Some(BackgroundsOption {
            label: "Show backgrounds inside each scenario".into(),
            inline: true,
            enabled: true,
        })
    );
}

#[test]
fn linked_backgrounds_are_written_so_and_a_background_without_steps_adds_nothing_and_disables_the_option()
 {
    let linked = PageOptions {
        inline_backgrounds: false,
        ..PageOptions::default()
    };
    let linked = page(FEATURE, &linked);
    assert!(!linked.blocks[1].backgrounds[0].inline);
    assert_eq!(linked.backgrounds_option.map(|o| o.inline), Some(false));
    let empty = page(
        "Feature: F\n  Background:\n    Only words.\n\n  Scenario: S\n    Given x\n",
        &PageOptions::default(),
    );
    assert!(empty.blocks[1].backgrounds.is_empty());
    assert_eq!(empty.backgrounds_option.map(|o| o.enabled), Some(false));
    let alone = page(
        "Feature: F\n  Background:\n    Given x\n",
        &PageOptions::default(),
    );
    assert_eq!(alone.backgrounds_option.map(|o| o.enabled), Some(false));
    let none = page(
        "Feature: F\n  Scenario: S\n    Given x\n",
        &PageOptions::default(),
    );
    assert_eq!(none.backgrounds_option, None);
}
