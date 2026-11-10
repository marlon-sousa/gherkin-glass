//! Policy: the spoken language a feature file is written in, as Gherkin names it.

/// The language of a file without a `# language:` line.
const DEFAULT_LANGUAGE: &str = "en";

/// Read as the `gherkin` crate reads it: the first line that is not blank,
/// when it is `# language: <name>`, with or without spaces around its parts.
pub(crate) fn file_language(source: &str) -> String {
    source
        .lines()
        .find(|line| !line.trim().is_empty())
        .and_then(|line| {
            let rest = line.trim().strip_prefix('#')?.trim_start();
            let rest = rest.strip_prefix("language")?.trim_start();
            Some(rest.strip_prefix(':')?.trim().to_string())
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
        assert_eq!(file_language("Feature: X\n"), "en");
        assert_eq!(file_language("# a comment\n# language: pt\n"), "en");
        assert_eq!(file_language(""), "en");
    }
}
