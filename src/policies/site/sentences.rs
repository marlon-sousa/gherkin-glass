//! Policy: the step sentences, the sentences of the suite, grouped and sorted, and their page.

mod sentences_page_building;
mod step_sentences;

pub(super) use sentences_page_building::build_sentences_page;

use step_sentences::find_step_sentences;
