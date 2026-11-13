use super::*;

#[test]
fn headings_read_the_keyword_and_the_name_at_their_levels() {
    let page = page(FEATURE, &PageOptions::default());
    assert_eq!(page.page.heading, "Feature: Refunds");
    assert_eq!(
        page.page.title,
        "Feature: Refunds, billing/saque em conta.feature"
    );
    assert_eq!(page.page.address.as_str(), "billing/saque em conta.html");
    let (background, scenario, rule) = blocks(&page);
    let top: Vec<_> = [
        &background.block.heading,
        &scenario.block.heading,
        &rule.block.heading,
    ]
    .into_iter()
    .map(|h| (h.level.number(), h.text(), h.anchor.as_str()))
    .collect();
    assert_eq!(
        top,
        [
            (2, "Background".to_string(), "background"),
            (2, "Scenario: Outside".to_string(), "outside"),
            (2, "Rule: R".to_string(), "r"),
        ]
    );
    let (background, outline) = in_rule(rule);
    let inside: Vec<_> = [&background.block.heading, &outline.block.heading]
        .into_iter()
        .map(|h| (h.level.number(), h.anchor.as_str()))
        .collect();
    assert_eq!(inside, [(3, "in-the-rule"), (3, "inside")]);
    let examples = &outline.examples[0];
    assert_eq!(
        (
            examples.heading.level.number(),
            examples.heading.text().as_str()
        ),
        (4, "Examples")
    );
    assert_eq!(
        examples.table,
        Some(TableView {
            header: vec!["n".into()],
            rows: vec![vec!["1".into()]],
        })
    );
}

#[test]
fn an_outline_shows_its_rows_as_scenarios_in_a_collapsed_block() {
    let page = page(FEATURE, &PageOptions::default());
    let (_, outline) = in_rule(blocks(&page).2);
    let section = outline.as_scenarios.as_ref().expect("rows");
    let heading = &section.heading;
    assert_eq!(
        (
            heading.level.number(),
            heading.anchor.as_str(),
            heading.text().as_str(),
            heading.collapsed
        ),
        (4, "inside_scenarios", "Examples as scenarios: Inside", true)
    );
    let row = &section.scenarios[0];
    assert_eq!(
        (
            row.heading.level.number(),
            row.heading.anchor.as_str(),
            row.heading.text().as_str()
        ),
        (5, "inside-row-1", "Inside (n: 1)")
    );
    assert_eq!(row.steps[0].step.parts, [StepPart::Number("1".into())]);
    let isolated: Vec<_> = heading
        .parts
        .iter()
        .map(|part| (part.text.as_str(), part.isolated))
        .collect();
    assert_eq!(
        isolated,
        [("Examples as scenarios: ", false), ("Inside", true)]
    );
    let isolated: Vec<_> = row
        .heading
        .parts
        .iter()
        .map(|part| (part.text.as_str(), part.isolated))
        .collect();
    assert_eq!(
        isolated,
        [
            ("Inside", true),
            (" (", false),
            ("n", true),
            (": ", false),
            ("1", true),
            (")", false)
        ]
    );
    let scenarios = PageOptions {
        collapse_scenarios: true,
        ..PageOptions::default()
    };
    let page = self::page(FEATURE, &scenarios);
    let (_, outline) = in_rule(blocks(&page).2);
    let as_scenarios = outline.as_scenarios.as_ref().expect("rows");
    assert!(as_scenarios.scenarios[0].heading.collapsed);
}

#[test]
fn the_settings_choose_which_blocks_start_collapsed() {
    let collapsed = |options: &PageOptions| {
        let page = page(FEATURE, options);
        let (background, scenario, rule) = blocks(&page);
        let (rule_background, outline) = in_rule(rule);
        [
            &background.block.heading,
            &scenario.block.heading,
            &rule.block.heading,
            &rule_background.block.heading,
            &outline.block.heading,
            &outline.examples[0].heading,
        ]
        .map(|heading| heading.collapsed)
    };
    // Rule 51: the inline form, the default, collapses the backgrounds.
    assert_eq!(
        collapsed(&PageOptions::default()),
        [true, false, false, true, false, false]
    );
    let linked = PageOptions {
        inline_backgrounds: false,
        ..PageOptions::default()
    };
    assert_eq!(collapsed(&linked), [false; 6]);
    let rules = PageOptions {
        collapse_rules: true,
        ..linked.clone()
    };
    assert_eq!(collapsed(&rules), [false, false, true, false, false, false]);
    let scenarios = PageOptions {
        collapse_scenarios: true,
        ..linked
    };
    assert_eq!(
        collapsed(&scenarios),
        [true, true, false, true, true, false]
    );
}
