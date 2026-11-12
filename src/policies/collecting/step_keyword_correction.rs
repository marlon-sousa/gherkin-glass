//! Policy: each step's keyword with the space the file writes after it, which the `gherkin` crate drops when its language's keyword has none.

use gherkin::Feature;

use super::doc_string_correction::steps_mut;

/// A keyword the file follows with a space or a tab gets it back; the
/// keywords of most languages, such as `Given `, already end with one.
pub(crate) fn correct_step_keywords(feature: &mut Feature, source: &str) {
    for step in steps_mut(feature) {
        if step.keyword.ends_with([' ', '\t']) {
            continue;
        }
        let after = source
            .get(step.span.start..)
            .and_then(|rest| rest.strip_prefix(step.keyword.as_str()))
            .and_then(|rest| rest.chars().next());
        if let Some(space @ (' ' | '\t')) = after {
            step.keyword.push(space);
        }
    }
}

#[cfg(test)]
mod tests {
    use gherkin::{GherkinEnv, Step};

    use super::*;

    fn keywords(source: &str) -> Vec<String> {
        let mut feature = Feature::parse(source, GherkinEnv::default()).expect("parses");
        correct_step_keywords(&mut feature, source);
        let steps: Vec<&Step> = feature.scenarios[0].steps.iter().collect();
        steps.iter().map(|step| step.keyword.clone()).collect()
    }

    #[test]
    fn a_keyword_keeps_the_space_written_after_it() {
        assert_eq!(
            keywords("# language: em\n📚: E\n  📕: S\n    😐 something\n    😂something\n"),
            ["😐 ", "😂"]
        );
        assert_eq!(
            keywords("# language: zh-CN\n功能: F\n  场景: S\n    假如有\n"),
            ["假如"]
        );
        assert_eq!(
            keywords("Feature: F\n  Scenario: S\n    Given x\n"),
            ["Given "]
        );
    }
}
