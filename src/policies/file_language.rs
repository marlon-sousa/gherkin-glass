//! Policy: the spoken language a feature file is written in, as Gherkin names it.

/// The language of a file without a `# language:` line.
const DEFAULT_LANGUAGE: &str = "en";

/// The only characters the `gherkin` crate allows around the parts of the line.
const SPACE: [char; 2] = [' ', '\t'];

/// Read as the `gherkin` crate reads it: the first line that is not blank,
/// when it is `# language: <name>`, with or without spaces around its parts.
pub(crate) fn file_language(source: &str) -> String {
    source
        .lines()
        .find(|line| !line.trim_matches(SPACE).is_empty())
        .and_then(|line| {
            let rest = line.trim_matches(SPACE).strip_prefix('#')?;
            let rest = rest.trim_start_matches(SPACE).strip_prefix("language")?;
            let rest = rest.trim_start_matches(SPACE).strip_prefix(':')?;
            Some(rest.trim_start_matches(SPACE).to_string())
        })
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| DEFAULT_LANGUAGE.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_language_line_names_it_and_its_absence_is_english() {
        assert_eq!(file_language("# language: pt\nFuncionalidade: X\n"), "pt");
        assert_eq!(file_language("\n  #language:zh-TW  \r\n"), "zh-TW");
        assert_eq!(file_language("#\u{a0}language: pt\n"), "en");
        assert_eq!(file_language("Feature: X\n"), "en");
        assert_eq!(file_language("# a comment\n# language: pt\n"), "en");
        assert_eq!(file_language(""), "en");
    }
}
