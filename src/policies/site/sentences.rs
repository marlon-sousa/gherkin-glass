//! Policy: the step sentences, each step's sentence, the sentences of the suite, and their page.

mod sentence_slots;
mod sentences_page_building;
mod step_sentences;

pub(crate) use sentences_page_building::{SENTENCES_PAGE, build_sentences_page};
pub(crate) use step_sentences::find_step_sentences;
