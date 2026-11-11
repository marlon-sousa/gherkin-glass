use super::*;

#[test]
fn headings_read_the_keyword_and_the_name_at_their_levels() {
    let page = page(FEATURE, &PageOptions::default());
    assert_eq!(page.heading, "Feature: Refunds");
    assert_eq!(
        page.title,
        "Feature: Refunds, billing/saque em conta.feature"
    );
    assert_eq!(page.address.as_str(), "billing/saque em conta.html");
    let top: Vec<_> = page
        .blocks
        .iter()
        .map(|b| (b.level.number(), b.heading(), b.anchor.as_str()))
        .collect();
    assert_eq!(
        top,
        [
            (2, "Background".to_string(), "background"),
            (2, "Scenario: Outside".to_string(), "outside"),
            (2, "Rule: R".to_string(), "r"),
        ]
    );
    let rule = &page.blocks[2];
    let inside: Vec<_> = rule
        .blocks
        .iter()
        .map(|b| (b.level.number(), b.anchor.as_str()))
        .collect();
    assert_eq!(inside, [(3, "in-the-rule"), (3, "inside")]);
    let examples = &rule.blocks[1].blocks[0];
    assert_eq!(
        (examples.level.number(), examples.heading().as_str()),
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
    let outline = &page.blocks[2].blocks[1];
    let section = &outline.blocks[1];
    assert_eq!(
        (
            section.kind,
            section.level.number(),
            section.anchor.as_str(),
            section.heading().as_str(),
            section.collapsed
        ),
        (
            BlockKind::ExamplesAsScenarios,
            4,
            "inside_scenarios",
            "Examples as scenarios: Inside",
            true
        )
    );
    let row = &section.blocks[0];
    assert_eq!(
        (
            row.level.number(),
            row.anchor.as_str(),
            row.heading().as_str()
        ),
        (5, "inside-row-1", "Inside (n: 1)")
    );
    assert_eq!(row.steps[0].parts, [StepPart::Number("1".into())]);
    let isolated: Vec<_> = section
        .heading_parts
        .iter()
        .map(|part| (part.text.as_str(), part.isolated))
        .collect();
    assert_eq!(
        isolated,
        [("Examples as scenarios: ", false), ("Inside", true)]
    );
    let isolated: Vec<_> = row
        .heading_parts
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
    assert!(page.blocks[2].blocks[1].blocks[1].blocks[0].collapsed);
}

#[test]
fn the_settings_choose_which_blocks_start_collapsed() {
    let collapsed = |options: &PageOptions| {
        let page = page(FEATURE, options);
        let rule = &page.blocks[2];
        [
            page.blocks[0].collapsed,
            page.blocks[1].collapsed,
            rule.collapsed,
            rule.blocks[0].collapsed,
            rule.blocks[1].collapsed,
            rule.blocks[1].blocks[0].collapsed,
        ]
    };
    assert_eq!(collapsed(&PageOptions::default()), [false; 6]);
    let rules = PageOptions {
        collapse_rules: true,
        ..PageOptions::default()
    };
    assert_eq!(collapsed(&rules), [false, false, true, false, false, false]);
    let scenarios = PageOptions {
        collapse_scenarios: true,
        ..PageOptions::default()
    };
    assert_eq!(
        collapsed(&scenarios),
        [true, true, false, true, true, false]
    );
}
