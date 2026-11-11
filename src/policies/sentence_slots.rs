//! Policy: a step's sentence, with its quoted values and outline placeholders as slots.

use unicode_normalization::UnicodeNormalization;

/// How every slot is shown in a sentence.
const SLOT: &str = "\"…\"";

/// The sentence of a step's `text`, in NFC, and the text of each of its slots
/// in order, as written. `columns` are the names of the outline's columns,
/// and empty outside an outline.
pub(crate) fn sentence_of(text: &str, columns: &[&str]) -> (String, Vec<String>) {
    let columns: Vec<String> = columns.iter().map(|name| name.nfc().collect()).collect();
    let is_column = |name: &str| columns.contains(&name.nfc().collect());
    let mut sentence = String::with_capacity(text.len());
    let mut values = Vec::new();
    let mut rest = text;
    let mut after_backslash = false;
    while let Some(character) = rest.chars().next() {
        let slot = match character {
            '"' if !after_backslash => closing_quote(&rest[1..]).map(|end| end + 2),
            '<' => rest[1..]
                .find('>')
                .filter(|&end| end > 0 && is_column(&rest[1..end + 1]))
                .map(|end| end + 2),
            _ => None,
        };
        match slot {
            Some(length) => {
                sentence.push_str(SLOT);
                values.push(rest[..length].to_string());
                after_backslash = rest[..length].ends_with('\\');
                rest = &rest[length..];
            }
            None => {
                sentence.push(character);
                after_backslash = character == '\\';
                rest = &rest[character.len_utf8()..];
            }
        }
    }
    (sentence.nfc().collect(), values)
}

/// Where the first `"` of `text` not just after a backslash is, if any.
fn closing_quote(text: &str) -> Option<usize> {
    let mut after_backslash = false;
    text.char_indices().find_map(|(at, character)| {
        let found = character == '"' && !after_backslash;
        after_backslash = character == '\\';
        found.then_some(at)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_values_are_slots_and_numbers_are_text() {
        assert_eq!(
            sentence_of(r#"the user "ana" pays 10"#, &[]),
            (
                r#"the user "…" pays 10"#.to_string(),
                vec![r#""ana""#.to_string()]
            )
        );
        assert_eq!(
            sentence_of(r#"a "" and "b c""#, &[]),
            (
                r#"a "…" and "…""#.to_string(),
                vec![r#""""#.to_string(), r#""b c""#.to_string()]
            )
        );
    }

    #[test]
    fn a_quote_with_no_other_after_it_and_other_quotation_marks_are_text() {
        assert_eq!(
            sentence_of(r#"the user's card is "kept"#, &[]),
            (r#"the user's card is "kept"#.to_string(), vec![])
        );
        assert_eq!(
            sentence_of("it shows “hi” and 'yo'", &[]),
            ("it shows “hi” and 'yo'".to_string(), vec![])
        );
    }

    #[test]
    fn only_the_outline_s_columns_are_placeholders() {
        assert_eq!(
            sentence_of("the user <name> sees < 0 and <other>", &["name"]),
            (
                r#"the user "…" sees < 0 and <other>"#.to_string(),
                vec!["<name>".to_string()]
            )
        );
        assert_eq!(
            sentence_of("the user <name> logs in", &[]),
            ("the user <name> logs in".to_string(), vec![])
        );
        assert_eq!(
            sentence_of("<first name> and <a>", &["first name", "a"]),
            (
                r#""…" and "…""#.to_string(),
                vec!["<first name>".to_string(), "<a>".to_string()]
            )
        );
    }

    #[test]
    fn a_placeholder_inside_quotes_is_part_of_one_slot() {
        assert_eq!(
            sentence_of(r#"the user "<name>" exists"#, &["name"]),
            (
                r#"the user "…" exists"#.to_string(),
                vec![r#""<name>""#.to_string()]
            )
        );
    }

    #[test]
    fn the_sentence_is_in_nfc_and_its_values_as_written() {
        assert_eq!(
            sentence_of("cafe\u{301} \"cafe\u{301}\"", &[]),
            (
                "caf\u{e9} \"…\"".to_string(),
                vec!["\"cafe\u{301}\"".to_string()]
            )
        );
    }

    #[test]
    fn a_placeholder_matches_its_column_whatever_the_normalisation() {
        assert_eq!(
            sentence_of("a <cafe\u{301}> and <caf\u{e9}>", &["caf\u{e9}"]),
            (
                r#"a "…" and "…""#.to_string(),
                vec!["<cafe\u{301}>".to_string(), "<caf\u{e9}>".to_string()]
            )
        );
    }

    #[test]
    fn slots_are_found_from_the_left_and_the_first_to_open_wins() {
        assert_eq!(
            sentence_of(r#"the "<na"me>" and <me>"#, &["na\"me", "me"]),
            (
                r#"the "…"me>" and "…""#.to_string(),
                vec![r#""<na""#.to_string(), "<me>".to_string()]
            )
        );
    }

    #[test]
    fn a_quote_just_after_a_backslash_neither_opens_nor_closes_a_slot() {
        assert_eq!(
            sentence_of(r#"it says \"hi\" and "a \" b" and \\"c""#, &[]),
            (
                r#"it says \"hi\" and "…" and \\"c""#.to_string(),
                vec![r#""a \" b""#.to_string()]
            )
        );
    }

    #[test]
    fn an_empty_name_is_no_placeholder() {
        assert_eq!(
            sentence_of("a <> and <a>", &["", "a"]),
            (r#"a <> and "…""#.to_string(), vec!["<a>".to_string()])
        );
    }
}
