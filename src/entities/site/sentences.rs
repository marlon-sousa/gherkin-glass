//! Entity: the step sentences of the suite and their page.

mod sentences_page;
mod step_sentences;

pub(crate) use sentences_page::{
    GroupContent, GroupView, Piece, SentenceContent, SentenceView, SentencesPage,
    SentencesPageContent, UseView,
};
pub(crate) use step_sentences::{Sentence, SentenceGroup, SentenceUse, StepKind, StepSentences};
