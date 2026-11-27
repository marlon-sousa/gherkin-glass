//! Entity: the words of a block's heading in parts, feature text inside gherkin-glass's own isolated.

/// A part of a heading; feature text inside gherkin-glass's own words is isolated, with `<bdi>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HeadingPart {
    pub(crate) text: String,
    pub(crate) isolated: bool,
}

/// `text` as the only part of a heading, gherkin-glass's own or a name not inside its words,
/// not isolated.
pub(crate) fn plain_parts(text: &str) -> Vec<HeadingPart> {
    vec![HeadingPart {
        text: text.to_string(),
        isolated: false,
    }]
}

/// The text of a heading's `parts`, without their markup.
pub(crate) fn text_of(parts: &[HeadingPart]) -> String {
    parts.iter().map(|part| part.text.as_str()).collect()
}
