// Adapter: the markup of the page of search results, drawn from its view, in the elements and classes the stylesheet styles, with the shell's search box put under its `h1`.

import type { Markup } from "./jsx/jsx_runtime.ts";
import type { SearchPageView } from "../policies/search_page_view.ts";

/** The content of the page's `main`, `searchBox` moved into it. */
export function resultsPage(view: SearchPageView, searchBox: Node | null): Markup {
  const words = view.words;
  const headings = words.headings;
  return (
    <>
      <h1 class="ghg-search-heading">{view.heading}</h1>
      {searchBox}
      <p class="ghg-results-status" role="status" />
      <div
        class="ghg-search-groups"
        data-heading={words.heading}
        data-title={words.title}
        data-loading={words.loading}
        data-failed={words.failed}
        data-none={words.none}
        data-count={words.count}
        data-within={words.within}
        data-separator={words.separator}
        data-matching-steps={words.matchingSteps}
        data-matching-tags={words.matchingTags}
        data-matching-cells={words.matchingCells}
        data-feature={headings.feature}
        data-unnamed-feature={headings.unnamedFeature}
        data-rule={headings.rule}
        data-unnamed-rule={headings.unnamedRule}
        data-background={headings.background}
        data-unnamed-background={headings.unnamedBackground}
        data-scenario={headings.scenario}
        data-unnamed-scenario={headings.unnamedScenario}
        data-scenario-outline={headings.scenarioOutline}
        data-unnamed-scenario-outline={headings.unnamedScenarioOutline}
        data-examples={headings.examples}
        data-numbered-examples={headings.numberedExamples}
      />
    </>
  );
}
