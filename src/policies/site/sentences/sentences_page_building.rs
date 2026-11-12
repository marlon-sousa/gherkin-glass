//! Policy: the page of step sentences, from the sentences found and the features they are in.

use i18n_embed::LanguageLoader;
use i18n_embed_fl::fl;

use crate::entities::{
    GroupView, LinkAddress, PageOptions, Piece, SentenceUse, SentenceView, SentencesPage, StepKind,
    StepSentences, UseView,
};
use crate::policies::site::{breadcrumb, site_title};
use crate::policies::{CATALOGUE, SENTENCES_PAGE};

/// Stand for a message's placeholders, to find where each goes; no message
/// of the catalogue holds them.
const MARKS: [&str; 2] = ["\u{0}", "\u{1}"];

pub(crate) fn build_sentences_page(
    sentences: &StepSentences,
    options: &PageOptions,
) -> SentencesPage {
    let own = CATALOGUE.current_language().to_string();
    // A use's language is shown only where it differs from the page's own.
    let language_of = |found: &SentenceUse| (found.language != own).then(|| found.language.clone());
    let groups = sentences
        .groups
        .iter()
        .map(|group| GroupView {
            heading: kind_heading(group.kind),
            sentences: group
                .sentences
                .iter()
                .map(|sentence| SentenceView {
                    text: sentence.text.clone(),
                    language: sentence.uses.first().and_then(language_of),
                    uses_count: fl!(CATALOGUE, "sentence-uses", count = sentence.uses.len()),
                    uses: sentence
                        .uses
                        .iter()
                        .map(|found| use_view(found, language_of(found)))
                        .collect(),
                })
                .collect(),
        })
        .collect();
    let heading = fl!(CATALOGUE, "sentences-title");
    let site = site_title(options);
    SentencesPage {
        breadcrumb: breadcrumb(&site, &[], Some((&heading, SENTENCES_PAGE))),
        language: own.clone(),
        title: fl!(CATALOGUE, "sentences-page-title", site = site.as_str()),
        heading,
        groups,
        no_steps: fl!(CATALOGUE, "sentences-none"),
    }
}

fn use_view(found: &SentenceUse, language: Option<String>) -> UseView {
    let link = fl!(
        CATALOGUE,
        "sentence-use",
        heading = MARKS[0],
        path = MARKS[1]
    );
    let item = if found.values.is_empty() {
        Vec::new()
    } else {
        pieces(&fl!(
            CATALOGUE,
            "sentence-use-values",
            link = MARKS[0],
            values = MARKS[1]
        ))
    };
    UseView {
        address: LinkAddress::to(found.page.as_str()).at(&found.anchor),
        language,
        link: pieces(&link),
        item,
        heading: found.heading.clone(),
        path: found.path.clone(),
        values: found.values.clone(),
        separator: fl!(CATALOGUE, "sentence-values-separator"),
    }
}

fn kind_heading(kind: StepKind) -> String {
    match kind {
        StepKind::Given => fl!(CATALOGUE, "sentences-given"),
        StepKind::When => fl!(CATALOGUE, "sentences-when"),
        StepKind::Then => fl!(CATALOGUE, "sentences-then"),
    }
}

/// `words` cut at each of the marks, whatever order the message puts them in.
fn pieces(words: &str) -> Vec<Piece> {
    let mut pieces = Vec::new();
    let mut rest = words;
    while let Some((before, number, after)) = MARKS
        .iter()
        .enumerate()
        .filter_map(|(number, mark)| {
            rest.split_once(mark)
                .map(|(before, after)| (before, number, after))
        })
        .min_by_key(|(before, number, _)| (before.len(), *number))
    {
        if !before.is_empty() {
            pieces.push(Piece::Words(before.to_string()));
        }
        pieces.push(Piece::Placeholder(number));
        rest = after;
    }
    if !rest.is_empty() {
        pieces.push(Piece::Words(rest.to_string()));
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{Sentence, SentenceGroup, SiteAddress};

    #[test]
    fn a_message_is_cut_at_its_marks_in_the_order_it_puts_them() {
        assert_eq!(
            pieces("\u{1} then \u{0}."),
            [
                Piece::Placeholder(1),
                Piece::Words(" then ".to_string()),
                Piece::Placeholder(0),
                Piece::Words(".".to_string()),
            ]
        );
    }

    #[test]
    fn the_count_is_a_plural_form() {
        let page = |uses: usize| {
            let found = SentenceUse {
                path: "a.feature".to_string(),
                page: SiteAddress::of::<&str>(&[], "a.html"),
                language: "en".to_string(),
                anchor: "s".to_string(),
                heading: "Scenario: S".to_string(),
                line: 1,
                values: Vec::new(),
            };
            let sentences = StepSentences {
                groups: vec![SentenceGroup {
                    kind: StepKind::Given,
                    sentences: vec![Sentence {
                        text: "x".to_string(),
                        uses: vec![found; uses],
                    }],
                }],
            };
            build_sentences_page(&sentences, &PageOptions::default()).groups[0].sentences[0]
                .uses_count
                .clone()
        };
        assert_eq!(page(1), "Used once.");
        assert_eq!(page(2), "Used 2 times.");
    }

    #[test]
    fn with_no_step_there_is_no_group() {
        let page = build_sentences_page(&StepSentences::default(), &PageOptions::default());
        assert!(page.groups.is_empty());
        assert_eq!(page.no_steps, "No feature has a step.");
    }
}
