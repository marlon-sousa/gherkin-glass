//! Policy: the words of a block's heading, in parts, from a message of the catalogue with feature text in it.

use crate::entities::HeadingPart;

/// Stand for feature text in a heading's words, to find where it goes; no
/// message of the catalogue holds them.
pub(super) const TEXT_MARKS: [&str; 2] = ["\u{1}", "\u{2}"];

/// The keyword as written, then `: ` and the name, or the keyword alone.
pub(crate) fn heading(keyword: &str, name: &str) -> String {
    let keyword = keyword.trim();
    let name = name.trim();
    if name.is_empty() {
        keyword.to_string()
    } else {
        format!("{keyword}: {name}")
    }
}

/// `words`, worded with `TEXT_MARKS` in place of `texts`, in parts, each
/// mark replaced by its text's parts.
pub(super) fn worded_parts(words: &str, texts: &[Vec<HeadingPart>]) -> Vec<HeadingPart> {
    debug_assert!(texts.len() <= TEXT_MARKS.len());
    let mut parts = Vec::new();
    let mut rest = words;
    while let Some((before, text, after)) = first_mark(rest, texts) {
        parts.push(plain(before));
        parts.extend(text.iter().cloned());
        rest = after;
    }
    parts.push(plain(rest));
    parts.retain(|part| !part.text.is_empty());
    parts
}

/// The words before the first mark in `rest`, the text that goes in its place, and the words after it.
fn first_mark<'a, 'b>(
    rest: &'a str,
    texts: &'b [Vec<HeadingPart>],
) -> Option<(&'a str, &'b [HeadingPart], &'a str)> {
    TEXT_MARKS
        .iter()
        .zip(texts)
        .filter_map(|(mark, text)| split_at_mark(rest, mark, text))
        .min_by_key(|(before, _, _)| before.len())
}

fn split_at_mark<'a, 'b>(
    rest: &'a str,
    mark: &str,
    text: &'b [HeadingPart],
) -> Option<(&'a str, &'b [HeadingPart], &'a str)> {
    let (before, after) = rest.split_once(mark)?;
    Some((before, text, after))
}

pub(super) fn plain(text: &str) -> HeadingPart {
    HeadingPart {
        text: text.to_string(),
        isolated: false,
    }
}

pub(super) fn isolated(text: &str) -> Vec<HeadingPart> {
    vec![HeadingPart {
        text: text.to_string(),
        isolated: true,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worded_parts_put_each_text_where_its_mark_is_whatever_the_order() {
        let words = format!("{}, then {}.", TEXT_MARKS[1], TEXT_MARKS[0]);
        let parts = worded_parts(&words, &[isolated("a\u{1}"), isolated("b")]);
        let parts: Vec<_> = parts
            .iter()
            .map(|part| (part.text.as_str(), part.isolated))
            .collect();
        assert_eq!(
            parts,
            [
                ("b", true),
                (", then ", false),
                ("a\u{1}", true),
                (".", false)
            ]
        );
    }
}
