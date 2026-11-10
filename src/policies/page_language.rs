//! Policy: the language tag of a page, from the name Gherkin gives its file's language.

/// The names of Gherkin's languages that are not the language's tag, checked
/// against the names in the `gherkin` crate's `languages.json`, 0.16.0.
const NOT_TAGS: [(&str, &str); 8] = [
    ("am", "hy"),
    ("bm", "ms"),
    ("em", "und"),
    ("en-lol", "en"),
    ("en-old", "ang"),
    ("en-tx", "en"),
    ("gj", "gu"),
    ("lu", "lb"),
];

pub(crate) fn page_language(gherkin: &str) -> &str {
    NOT_TAGS
        .iter()
        .find(|(name, _)| *name == gherkin)
        .map_or(gherkin, |(_, tag)| tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_gherkin_name_is_its_tag_unless_it_is_not_a_tag() {
        assert_eq!(page_language("pt"), "pt");
        assert_eq!(page_language("zh-TW"), "zh-TW");
        assert_eq!(page_language("em"), "und");
        assert_eq!(page_language("gj"), "gu");
        assert_eq!(page_language("am"), "hy");
    }
}
