mod against_fixtures;

use gherkin::{Feature, GherkinEnv};

use super::*;
use StepPart::{Number, Parameter, Placeholder, Plain};

/// The step's text after its keyword, as its parts hold it.
fn text(step: &HighlightedStep) -> String {
    step.parts.iter().map(StepPart::text).collect()
}

/// The parts of `step`, as the only step of a scenario with examples whose
/// columns are `columns`, or of one without examples when there are none.
fn highlight(step: &str, columns: &[&str]) -> Vec<StepPart> {
    let mut source = format!("Feature: F\n  Scenario: S\n    Given {step}\n");
    if !columns.is_empty() {
        source += &format!(
            "    Examples:\n      | {} |\n      | {} |\n",
            columns.join(" | "),
            columns.iter().map(|_| "x").collect::<Vec<_>>().join(" | ")
        );
    }
    let feature = Feature::parse(source, GherkinEnv::default()).expect("parses");
    let scenario = &feature.scenarios[0];
    let highlighted = highlight_step(&scenario.steps[0], Some(scenario));
    assert_eq!(highlighted.keyword, "Given");
    assert_eq!(text(&highlighted), step, "the text is unchanged");
    highlighted.parts
}

fn plain(text: &str) -> StepPart {
    Plain(text.into())
}

#[test]
fn plain_text_is_one_part() {
    assert_eq!(highlight("a customer", &[]), [plain("a customer")]);
}

#[test]
fn a_quoted_parameter_keeps_its_quotes() {
    assert_eq!(
        highlight(r#"the user types "hello 42" twice"#, &[]),
        [
            plain("the user types "),
            Parameter(vec![plain("\"hello 42\"")]),
            plain(" twice"),
        ]
    );
    assert_eq!(
        highlight(r#"an empty "" and "a \"b\"" end"#, &[]),
        [
            plain("an empty "),
            Parameter(vec![plain("\"\"")]),
            plain(" and "),
            Parameter(vec![plain(r#""a \"b\"""#)]),
            plain(" end"),
        ]
    );
}

#[test]
fn an_unclosed_or_escaped_quote_is_plain_and_single_quotes_are_not_parameters() {
    assert_eq!(
        highlight(r#"a "lone quote"#, &[]),
        [plain(r#"a "lone quote"#)]
    );
    assert_eq!(highlight(r#"a \"b" c"#, &[]), [plain(r#"a \"b" c"#)]);
    assert_eq!(
        highlight("the user's 'name'", &[]),
        [plain("the user's 'name'")]
    );
}

#[test]
fn numbers_stand_apart_from_words() {
    assert_eq!(
        highlight("pay 10, -3.5 and 1.000,50 or $7 on 2026-11-10.", &[]),
        [
            plain("pay "),
            Number("10".into()),
            plain(", "),
            Number("-3.5".into()),
            plain(" and "),
            Number("1.000,50".into()),
            plain(" or $"),
            Number("7".into()),
            plain(" on "),
            Number("2026".into()),
            plain("-"),
            Number("11".into()),
            plain("-"),
            Number("10".into()),
            plain("."),
        ]
    );
    assert_eq!(
        highlight("R2D2 is 3rd in v1.2 and x_1, item-4", &[]),
        [
            plain("R2D2 is 3rd in v1.2 and x_1, item-"),
            Number("4".into()),
        ]
    );
    assert_eq!(highlight("٣ apples", &[]), [plain("٣ apples")]);
    assert_eq!(
        highlight("2.x, 10.5.x, 1,a and e\u{301}5 or 5\u{301}", &[]),
        [plain("2.x, 10.5.x, 1,a and e\u{301}5 or 5\u{301}")]
    );
}

#[test]
fn a_placeholder_names_a_column_of_the_outline_s_examples() {
    assert_eq!(
        highlight(
            "<amount> of <currency> in <unknown> < 5 >",
            &["amount", "currency"]
        ),
        [
            Placeholder("<amount>".into()),
            plain(" of "),
            Placeholder("<currency>".into()),
            plain(" in <unknown> < "),
            Number("5".into()),
            plain(" >"),
        ]
    );
    assert_eq!(highlight("<> and <x>", &["", "x"])[0], plain("<> and "));
    assert_eq!(
        highlight("<a<amount>>", &["amount"]),
        [plain("<a"), Placeholder("<amount>".into()), plain(">")]
    );
}

#[test]
fn a_placeholder_inside_a_parameter_is_highlighted_and_a_number_is_not() {
    assert_eq!(
        highlight(r#"types "<name> 3" now"#, &["name"]),
        [
            plain("types "),
            Parameter(vec![
                plain("\""),
                Placeholder("<name>".into()),
                plain(" 3\""),
            ]),
            plain(" now"),
        ]
    );
}

#[test]
fn a_step_outside_an_outline_has_no_placeholders() {
    let feature = Feature::parse(
        "Feature: F\n  Background:\n    Given <amount>\n  Scenario: S\n    Given <amount>\n",
        GherkinEnv::default(),
    )
    .expect("parses");
    let background = &feature.background.as_ref().expect("a background").steps[0];
    assert_eq!(highlight_step(background, None).parts, [plain("<amount>")]);
    let scenario = &feature.scenarios[0];
    assert_eq!(
        highlight_step(&scenario.steps[0], Some(scenario)).parts,
        [plain("<amount>")]
    );
}

#[test]
fn a_word_with_a_point_or_a_mark_in_it_has_no_number() {
    assert_eq!(
        highlight("2.x, 10.5.x, 1,a and e\u{301}5 or 5\u{301}", &[]),
        [plain("2.x, 10.5.x, 1,a and e\u{301}5 or 5\u{301}")]
    );
}

#[test]
fn a_quote_after_a_backslash_never_closes_a_parameter() {
    assert_eq!(
        highlight(r#"x "a\\" b" c"#, &[]),
        [
            plain("x "),
            Parameter(vec![plain(r#""a\\" b""#)]),
            plain(" c")
        ]
    );
}

#[test]
fn an_empty_name_is_no_placeholder() {
    assert_eq!(
        highlight("<> and <x>", &["", "x"]),
        [plain("<> and "), Placeholder("<x>".into())]
    );
}
