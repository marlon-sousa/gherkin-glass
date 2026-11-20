//! Policy: the patterns of `keep_in_output` in a configuration file, a list of text in the gitignore syntax.

use toml::Spanned;
use toml::de::DeValue;

use super::error_at;
use crate::entities::{KeptInOutput, KeptPattern};
use crate::errors::{ConfigError, ConfigErrorKind};

const KEY: &str = "keep_in_output";

/// Every pattern of `keep_in_output`, or every error in them, each at the line of its pattern.
pub(super) fn kept_patterns(
    source: &str,
    value: &Spanned<DeValue<'_>>,
) -> Result<KeptInOutput, Vec<ConfigError>> {
    let not_list = || {
        vec![error_at(
            source,
            value.span().start,
            ConfigErrorKind::NotTextList,
        )]
    };
    let entries = value.get_ref().as_array().ok_or_else(not_list)?;
    let texts: Vec<_> = entries
        .iter()
        .map(text_at)
        .collect::<Option<_>>()
        .ok_or_else(not_list)?;
    let (patterns, failed): (Vec<_>, Vec<_>) = texts
        .into_iter()
        .map(|entry| pattern_at(source, entry))
        .partition(Result::is_ok);
    if failed.is_empty() {
        Ok(KeptInOutput::new(
            patterns.into_iter().filter_map(Result::ok).collect(),
        ))
    } else {
        Err(failed.into_iter().filter_map(Result::err).collect())
    }
}

/// The entry's offset and its text, when it is text.
fn text_at<'a>(entry: &'a Spanned<DeValue<'_>>) -> Option<(usize, &'a str)> {
    Some((entry.span().start, entry.get_ref().as_str()?))
}

/// The pattern at `start`, or what is wrong with it.
fn pattern_at(source: &str, (start, text): (usize, &str)) -> Result<KeptPattern, ConfigError> {
    kept_pattern(text).map_err(|kind| error_at(source, start, kind))
}

fn kept_pattern(text: &str) -> Result<KeptPattern, ConfigErrorKind> {
    if text.is_empty() {
        return Err(ConfigErrorKind::Empty {
            key: KEY.to_string(),
        });
    }
    KeptPattern::new(text).map_err(|reason| ConfigErrorKind::InvalidKeptPattern {
        pattern: text.to_string(),
        reason,
    })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::super::parse_config;
    use super::*;

    fn errors(source: &str) -> Vec<(Option<usize>, ConfigErrorKind)> {
        parse_config(source)
            .expect_err("problems")
            .into_iter()
            .map(|error| (error.line, error.kind))
            .collect()
    }

    // Rule 224.
    #[test]
    fn keep_in_output_is_read_in_order() {
        let config = parse_config("keep_in_output = [\".nojekyll\", \"/CNAME\"]\n").expect("read");
        let expected = [".nojekyll", "/CNAME"]
            .iter()
            .map(|text| KeptPattern::new(text).expect("valid"))
            .collect();
        assert_eq!(config.keep_in_output, Some(KeptInOutput::new(expected)));
    }

    // Rule 226.
    #[rstest]
    #[case::text("keep_in_output = \".nojekyll\"\n", 1, ConfigErrorKind::NotTextList)]
    #[case::a_list_holding_a_number(
        "keep_in_output = [\"a\", 1]\n",
        1,
        ConfigErrorKind::NotTextList
    )]
    #[case::an_empty_pattern(
        "keep_in_output = [\n  \"a\",\n  \"\",\n]\n",
        3,
        ConfigErrorKind::Empty { key: KEY.into() }
    )]
    #[case::a_pattern_that_is_not_valid(
        "keep_in_output = [\n  \"{draft\",\n]\n",
        2,
        ConfigErrorKind::InvalidKeptPattern {
            pattern: "{draft".into(),
            reason: KeptPattern::new("{draft").expect_err("not valid"),
        }
    )]
    fn a_mistake_in_keep_in_output_is_reported_on_its_line(
        #[case] source: &str,
        #[case] line: usize,
        #[case] kind: ConfigErrorKind,
    ) {
        assert_eq!(errors(source), [(Some(line), kind)]);
    }
}
