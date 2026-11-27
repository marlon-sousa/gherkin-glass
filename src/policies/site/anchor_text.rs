//! Policy: the anchor of a text, and an anchor made unique among those taken, by which a feature's blocks and the tags' pages are named.

use std::collections::HashSet;

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// `anchor`, or the first of it followed by `-2`, `-3` and so on that is not taken.
pub(super) fn unique(anchor: String, taken: &mut HashSet<String>) -> String {
    let mut candidate = anchor.clone();
    let mut number = 2;
    while taken.contains(&candidate) {
        candidate = format!("{anchor}-{number}");
        number += 1;
    }
    taken.insert(candidate.clone());
    candidate
}

/// The anchor of one name or keyword: lower-cased, without apostrophes, its letters and
/// digits kept with the marks and joiners inside its words, and one hyphen between words;
/// empty when nothing is left.
pub(super) fn anchor_of(text: &str) -> String {
    // Apostrophes go first, so that a mark after one joins the letter before it as NFC does;
    // lower-casing can leave text that is no longer in NFC, so it is normalised again.
    let lower: String = text
        .chars()
        .filter(|c| !matches!(c, '\'' | '\u{2019}'))
        .nfc()
        .collect::<String>()
        .to_lowercase()
        .nfc()
        .collect();
    let characters: Vec<char> = lower.chars().collect();
    let mut anchor = String::with_capacity(lower.len());
    // Whether the last character looked at was kept, so that a mark or a joiner may follow it.
    let mut joined = false;
    for (index, &character) in characters.iter().enumerate() {
        let keep = if character.is_alphanumeric() {
            if !joined && !anchor.is_empty() {
                anchor.push('-');
            }
            true
        } else if is_combining_mark(character) {
            joined
        } else if is_joiner(character) {
            joined
                && characters
                    .get(index + 1)
                    .is_some_and(|c| c.is_alphanumeric())
        } else {
            false
        };
        if keep {
            anchor.push(character);
        }
        joined = keep;
    }
    anchor
}

/// The zero width non-joiner and joiner, which Persian and the scripts of India write inside words.
fn is_joiner(character: char) -> bool {
    matches!(character, '\u{200c}' | '\u{200d}')
}

#[cfg(test)]
mod tests {
    use proptest::prelude::proptest;
    use unicode_normalization::is_nfc;

    use super::*;

    proptest! {
        // Rules 27 to 29: any text, of any characters, makes an anchor in NFC, with no
        // apostrophe, no hyphen at either end and never two together.
        #[test]
        fn any_text_makes_an_anchor_of_words_joined_by_one_hyphen(text in "(?s).{0,40}|[Aa'’ 0-9-\u{301}\u{200c}\u{200d}\u{fe0f}çÉİ]{0,40}") {
            let anchor = anchor_of(&text);
            assert!(is_nfc(&anchor), "{text:?}: {anchor:?}");
            assert!(!anchor.contains(['\'', '\u{2019}']), "{text:?}: {anchor:?}");
            assert!(!anchor.starts_with('-') && !anchor.ends_with('-'), "{text:?}: {anchor:?}");
            assert!(!anchor.contains("--"), "{text:?}: {anchor:?}");
        }
    }

    #[test]
    fn a_name_is_lower_cased_and_its_gaps_become_one_hyphen() {
        assert_eq!(
            anchor_of("Withdraw  Cash, quickly!"),
            "withdraw-cash-quickly"
        );
        assert_eq!(anchor_of("  -- The end --  "), "the-end");
        assert_eq!(anchor_of("Step 2: pay"), "step-2-pay");
    }

    #[test]
    fn apostrophes_are_removed_not_turned_into_hyphens() {
        assert_eq!(anchor_of("The customer's card"), "the-customers-card");
        assert_eq!(anchor_of("The customer’s card"), "the-customers-card");
    }

    // Rules 27 and 28: a mark after an apostrophe joins the letter before it, in NFC.
    #[test]
    fn a_mark_after_an_apostrophe_joins_the_letter_before_it_in_nfc() {
        assert_eq!(anchor_of("c'\u{301}"), "\u{107}");
    }

    #[test]
    fn letters_outside_a_to_z_are_kept_in_nfc() {
        assert_eq!(
            anchor_of("Saque em conta poupança"),
            "saque-em-conta-poupança"
        );
        // "É" written as E and a combining acute accent.
        assert_eq!(anchor_of("E\u{301}TAPE"), "\u{e9}tape");
        assert_eq!(anchor_of("Straße"), "straße");
        assert_eq!(anchor_of("हिन्दी में"), "हिन्दी-में");
        assert_eq!(anchor_of("می\u{200c}خواهم"), "می\u{200c}خواهم");
    }

    #[test]
    fn a_mark_or_joiner_with_no_letter_before_it_is_a_gap() {
        // An emoji followed by the variation selector U+FE0F, as phones type it.
        assert_eq!(anchor_of("\u{2764}\u{fe0f} love"), "love");
        assert_eq!(anchor_of("\u{2764}\u{fe0f}"), "");
        assert_eq!(anchor_of("a \u{301}b"), "a-b");
        assert_eq!(anchor_of("a\u{200d} b"), "a-b");
        assert_eq!(anchor_of("\u{200c}a"), "a");
    }

    #[test]
    fn a_name_of_nothing_but_gaps_leaves_nothing() {
        assert_eq!(anchor_of("???"), "");
        assert_eq!(anchor_of("📕"), "");
    }

    #[test]
    fn repeats_take_the_first_free_number() {
        let mut taken = HashSet::new();
        let made: Vec<_> = ["a", "a", "a-2", "a"]
            .into_iter()
            .map(|anchor| unique(anchor.to_string(), &mut taken))
            .collect();
        assert_eq!(made, ["a", "a-2", "a-2-2", "a-3"]);
    }
}
