// Policy: the view of the page of search results a shell draws, its words for the script that lists the results, and its title, as docs/BEHAVIOUR.md, rules 216 to 218, give them.

import { searchedHeading } from "./result_groups.ts";
import type { BlockKind } from "../ports/store.ts";
import type { KindHeadings, PageWords } from "../ports/search_page.ts";
import type { Words } from "../ports/words.ts";

export interface SearchPageView {
  /** The page's title, which names the words searched for when there are any. */
  readonly title: string;
  /** The `h1` and the breadcrumb's words, until the results are listed. */
  readonly heading: string;
  readonly words: PageWords;
}

/** The page of search results for `searched`, the words of its address, in the site called `site`. */
export function searchPageView(words: Words, site: string, searched: string): SearchPageView {
  const pageWords: PageWords = {
    heading: words.text("search-page-heading-for", { words: "{words}" }),
    title: words.text("search-page-title-for", { words: "{words}", site }),
    loading: words.text("search-loading"),
    failed: words.text("search-failed"),
    none: words.text("search-none"),
    count: words.text("search-page-count", { count: "{count}", features: "{features}" }),
    within: words.text("search-page-within", { blocks: "{blocks}" }),
    separator: words.text("search-page-separator"),
    matchingSteps: words.text("search-page-matching-steps"),
    matchingTags: words.text("search-page-matching-tags"),
    matchingCells: words.text("search-page-matching-cells"),
    headings: kindHeadings(words),
  };
  const text = searched.trim();
  return {
    title: text === "" ? words.text("search-page-title", { site }) : searchedHeading(pageWords, text).title,
    heading: words.text("search-page-heading"),
    words: pageWords,
  };
}

function kindHeadings(words: Words): KindHeadings {
  const named = (kind: BlockKind): string => words.text("named-heading", { kind, name: "{name}" });
  const unnamed = (kind: BlockKind): string => words.text("unnamed-heading", { kind });
  return {
    feature: named("feature"),
    unnamedFeature: unnamed("feature"),
    rule: named("rule"),
    unnamedRule: unnamed("rule"),
    background: named("background"),
    unnamedBackground: unnamed("background"),
    scenario: named("scenario"),
    unnamedScenario: unnamed("scenario"),
    scenarioOutline: named("scenario-outline"),
    unnamedScenarioOutline: unnamed("scenario-outline"),
    examples: named("examples"),
    numberedExamples: words.text("numbered-examples", { place: "{place}" }),
  };
}
