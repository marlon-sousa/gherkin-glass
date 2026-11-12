use super::*;
use crate::entities::{SiteNode, SiteTreeNode};

/// Each background the scenario `node` starts with: its introduction, its
/// anchor, its heading and its first step's text, a `|` between them.
fn from(node: SiteTreeNode<'_>) -> Vec<String> {
    node.backgrounds()
        .into_iter()
        .map(|(background, background_node)| {
            let step = background_node
                .children()
                .find_map(|child| match child.value() {
                    SiteNode::Step(step) => Some(step.parts.clone()),
                    _ => None,
                });
            let [StepPart::Plain(text)] = step.as_deref().unwrap_or_default() else {
                panic!("{step:?}");
            };
            format!(
                "{}|{}|{}|{text}",
                background.introduction,
                background.heading.anchor,
                background.heading.text()
            )
        })
        .collect()
}

/// The scenarios of the page in `builder`, those inside its rules among them, in file order.
fn scenarios(builder: &SiteTreeBuilder) -> Vec<SiteTreeNode<'_>> {
    let page = builder
        .input_folder()
        .feature_pages()
        .next()
        .expect("the page");
    page.blocks()
        .flat_map(|block| [block].into_iter().chain(block.children()))
        .filter(|node| matches!(node.value(), SiteNode::Scenario(_)))
        .collect()
}

#[test]
fn each_scenario_finds_the_feature_s_background_then_its_rule_s_above_it() {
    let page = page(FEATURE, &PageOptions::default());
    assert_eq!(
        page.page.backgrounds_option,
        Some(BackgroundsOption {
            label: "Show backgrounds inside each scenario".into(),
            inline: true,
            enabled: true,
        })
    );
    let builder = tree_of(page);
    let [scenario, outline] = scenarios(&builder)[..] else {
        panic!("two scenarios");
    };
    let feature = "From the background:|background|Background|a customer";
    assert_eq!(from(scenario), [feature]);
    assert_eq!(
        from(outline),
        [
            feature,
            "From the rule's background:|in-the-rule|Background: In the rule|y"
        ]
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
    assert_eq!(
        linked.page.backgrounds_option.map(|o| o.inline),
        Some(false)
    );
    let empty = page(
        "Feature: F\n  Background:\n    Only words.\n\n  Scenario: S\n    Given x\n",
        &PageOptions::default(),
    );
    assert_eq!(
        empty.page.backgrounds_option.as_ref().map(|o| o.enabled),
        Some(false)
    );
    let builder = tree_of(empty);
    assert!(scenarios(&builder)[0].backgrounds().is_empty());
    let alone = page(
        "Feature: F\n  Background:\n    Given x\n",
        &PageOptions::default(),
    );
    assert_eq!(
        alone.page.backgrounds_option.map(|o| o.enabled),
        Some(false)
    );
    let in_a_rule = page(
        "Feature: F\n  Rule: R\n    Background:\n      Given x\n    Scenario: S\n      Given y\n",
        &PageOptions::default(),
    );
    assert_eq!(
        in_a_rule.page.backgrounds_option.map(|o| o.enabled),
        Some(true)
    );
    let none = page(
        "Feature: F\n  Scenario: S\n    Given x\n",
        &PageOptions::default(),
    );
    assert_eq!(none.page.backgrounds_option, None);
}
