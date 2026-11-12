use super::*;

#[test]
fn each_scenario_starts_with_the_feature_s_background_then_its_rule_s() {
    let page = page(FEATURE, &PageOptions::default());
    fn from(scenario: &ScenarioBlock) -> Vec<(&str, &str, &str, &str, bool)> {
        scenario
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
    let (_, scenario, rule) = blocks(&page);
    assert_eq!(from(scenario), [feature]);
    let (_, outline) = in_rule(rule);
    assert_eq!(
        from(outline),
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
    assert!(!blocks(&linked).1.backgrounds[0].inline);
    assert_eq!(linked.backgrounds_option.map(|o| o.inline), Some(false));
    let empty = page(
        "Feature: F\n  Background:\n    Only words.\n\n  Scenario: S\n    Given x\n",
        &PageOptions::default(),
    );
    let [_, FeatureBlock::Scenario(scenario)] = empty.blocks.as_slice() else {
        panic!("{:?}", empty.blocks);
    };
    assert!(scenario.backgrounds.is_empty());
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
