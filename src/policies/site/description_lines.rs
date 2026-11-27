//! Policy: a block's description as its page and its object show it, the lines that hold text without the spaces at their ends.

/// The lines of `text` that hold text, each without the spaces at its end; none for none.
pub(crate) fn description_lines(text: Option<&str>) -> Vec<String> {
    text.unwrap_or_default()
        .lines()
        .map(str::trim_end)
        .filter(has_text)
        .map(str::to_string)
        .collect()
}

fn has_text(line: &&str) -> bool {
    !line.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Rule 45: a description's lines, without their trailing spaces and its blank lines.
    #[test]
    fn a_description_keeps_its_lines_of_text() {
        assert_eq!(
            description_lines(Some("A refund,   \n\n  \nin cash.")),
            ["A refund,", "in cash."]
        );
        assert!(description_lines(None).is_empty());
    }
}
