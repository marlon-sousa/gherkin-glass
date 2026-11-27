// The words a shell gives the search box, read from the shell of tests/fixtures/pages, those the page of search
// results gives its script, read from the one committed in script/test/expected/pages/search.html, and the documents
// of tests/fixtures/search/search.json.

import { readFileSync } from "node:fs";

import { Window } from "happy-dom";

import type { SearchDocument } from "../../src/ports/documents.ts";
import type { SearchWords } from "../../src/ports/search_box.ts";
import type { PageWords } from "../../src/ports/search_page.ts";

const fixtures = new URL("../../../tests/fixtures/", import.meta.url);
const expected = new URL("../expected/", import.meta.url);

/** The text of each `data-` attribute of the first element of `page` that `element` selects, such as `form.ghg-search`. */
function elementData(page: URL, element: string): (name: string) => string {
  const window = new Window({ url: "https://example.com/site/" });
  window.document.write(readFileSync(page, "utf8"));
  const found = window.document.querySelector(element);
  return (name) => {
    const value = found?.getAttribute(`data-${name}`);
    if (value === null || value === undefined) {
      throw new Error(`${page.href} has no data-${name}`);
    }
    return value;
  };
}

export function pageWords(): SearchWords {
  const data = elementData(new URL("pages/expected/index.html", fixtures), "form.ghg-search");
  return {
    shown: Number(data("shown")),
    loading: data("loading"),
    failed: data("failed"),
    none: data("none"),
    count: data("count"),
    countLimited: data("count-limited"),
    noName: data("no-name"),
    feature: data("feature"),
    rule: data("rule"),
    scenario: data("scenario"),
    scenarioInRule: data("scenario-in-rule"),
    scenarioOutline: data("scenario-outline"),
    scenarioOutlineInRule: data("scenario-outline-in-rule"),
    background: data("background"),
    backgroundInRule: data("background-in-rule"),
    examples: data("examples"),
    examplesInRule: data("examples-in-rule"),
    numberedExamples: data("numbered-examples"),
    allResults: data("all-results"),
  };
}

export function searchPageWords(): PageWords {
  const data = elementData(new URL("pages/search.html", expected), "div.ghg-search-groups");
  return {
    heading: data("heading"),
    title: data("title"),
    loading: data("loading"),
    failed: data("failed"),
    none: data("none"),
    count: data("count"),
    within: data("within"),
    separator: data("separator"),
    matchingSteps: data("matching-steps"),
    matchingTags: data("matching-tags"),
    matchingCells: data("matching-cells"),
    headings: {
      feature: data("feature"),
      unnamedFeature: data("unnamed-feature"),
      rule: data("rule"),
      unnamedRule: data("unnamed-rule"),
      background: data("background"),
      unnamedBackground: data("unnamed-background"),
      scenario: data("scenario"),
      unnamedScenario: data("unnamed-scenario"),
      scenarioOutline: data("scenario-outline"),
      unnamedScenarioOutline: data("unnamed-scenario-outline"),
      examples: data("examples"),
      numberedExamples: data("numbered-examples"),
    },
  };
}

export function documents(): SearchDocument[] {
  return JSON.parse(readFileSync(new URL("search/search.json", fixtures), "utf8")) as SearchDocument[];
}
