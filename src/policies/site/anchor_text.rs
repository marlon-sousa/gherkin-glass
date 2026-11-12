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
    // Lower-casing can leave text that is no longer in NFC, so it is normalised again.
    let lower: String = text
        .nfc()
        .collect::<String>()
        .to_lowercase()
        .nfc()
        .collect();
    let characters: Vec<char> = lower
        .chars()
        .filter(|c| !matches!(c, '\'' | '\u{2019}'))
        .collect();
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
    use super::*;

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
