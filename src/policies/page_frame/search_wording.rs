//! Policy: the words of the search box, with marks where the script puts what it found.

use i18n_embed_fl::fl;

use crate::entities::SearchWords;
use crate::policies::CATALOGUE;

const SHOWN_RESULTS: usize = 50;

pub(crate) fn search_words() -> SearchWords {
    SearchWords {
        shown: SHOWN_RESULTS,
        label: fl!(CATALOGUE, "search-label"),
        loading: fl!(CATALOGUE, "search-loading"),
        failed: fl!(CATALOGUE, "search-failed"),
        none: fl!(CATALOGUE, "search-none"),
        count: fl!(CATALOGUE, "search-count", count = "{count}"),
        count_limited: fl!(
            CATALOGUE,
            "search-count-limited",
            count = "{count}",
            shown = SHOWN_RESULTS
        ),
        no_name: fl!(CATALOGUE, "search-no-name"),
        feature: fl!(CATALOGUE, "search-result-feature", name = "{name}"),
        rule: fl!(
            CATALOGUE,
            "search-result-rule",
            name = "{name}",
            feature = "{feature}"
        ),
        scenario: fl!(
            CATALOGUE,
            "search-result-scenario",
            name = "{name}",
            feature = "{feature}"
        ),
        scenario_in_rule: fl!(
            CATALOGUE,
            "search-result-scenario-in-rule",
            name = "{name}",
            feature = "{feature}",
            rule = "{rule}"
        ),
        scenario_outline: fl!(
            CATALOGUE,
            "search-result-scenario-outline",
            name = "{name}",
            feature = "{feature}"
        ),
        scenario_outline_in_rule: fl!(
            CATALOGUE,
            "search-result-scenario-outline-in-rule",
            name = "{name}",
            feature = "{feature}",
            rule = "{rule}"
        ),
        background: fl!(
            CATALOGUE,
            "search-result-background",
            name = "{name}",
            feature = "{feature}"
        ),
        background_in_rule: fl!(
            CATALOGUE,
            "search-result-background-in-rule",
            name = "{name}",
            feature = "{feature}",
            rule = "{rule}"
        ),
        examples: fl!(
            CATALOGUE,
            "search-result-examples",
            name = "{name}",
            feature = "{feature}",
            outline = "{outline}"
        ),
        examples_in_rule: fl!(
            CATALOGUE,
            "search-result-examples-in-rule",
            name = "{name}",
            feature = "{feature}",
            rule = "{rule}",
            outline = "{outline}"
        ),
        numbered_examples: fl!(CATALOGUE, "numbered-examples", place = "{place}"),
        all_results: fl!(CATALOGUE, "search-all-results", words = "{words}"),
        results: fl!(CATALOGUE, "search-page-heading"),
    }
}
