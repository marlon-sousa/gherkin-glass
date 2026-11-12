//! Policy: the step sentences, each step's sentence, the sentences of the suite, and their page.

mod sentence_slots;
mod sentences_page_building;
mod step_sentences;

pub(super) use sentences_page_building::build_sentences_page;

use step_sentences::find_step_sentences;
