//! Policy: whether a feature file writes a block with another of Gherkin's words than its kind's, which a feature's page notes and the store records.

use i18n_embed_fl::fl;

use crate::entities::BlockKind;
use crate::policies::CATALOGUE;

/// Whether `keyword`, as the file writes it, differs from the words of `kind`
/// other than in the case of its letters.
pub(crate) fn writes_another_keyword(kind: BlockKind, keyword: &str) -> bool {
    let kind_words = fl!(CATALOGUE, "unnamed-heading", kind = kind.as_str());
    keyword.trim().to_lowercase() != kind_words.to_lowercase()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::the_kind_s_word(BlockKind::Scenario, "Scenario", false)]
    #[case::in_another_case(BlockKind::ScenarioOutline, "Scenario Outline", false)]
    #[case::another_word(BlockKind::Scenario, "Example", true)]
    #[case::another_language(BlockKind::Feature, "Funcionalidade", true)]
    fn a_keyword_is_another_word_when_it_differs_beyond_its_case(
        #[case] kind: BlockKind,
        #[case] keyword: &str,
        #[case] another: bool,
    ) {
        assert_eq!(writes_another_keyword(kind, keyword), another);
    }
}
