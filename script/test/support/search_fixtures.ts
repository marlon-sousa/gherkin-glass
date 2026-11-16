// The words a page gives the search box, read from tests/fixtures/pages/expected/stories/read.html,
// those the page of search results gives its script, read from tests/fixtures/pages/expected/search.html,
// and the documents of tests/fixtures/search/search.json.

import { readFileSync } from "node:fs";

import type { SearchDocument } from "../../src/ports/documents.ts";
import type { SearchWords } from "../../src/ports/search_box.ts";
import type { PageWords } from "../../src/ports/search_page.ts";

const fixtures = new URL("../../../tests/fixtures/", import.meta.url);

/** The text of each `data-` attribute of the first element of `page` whose tag and class are `element`, such as `form.ghg-search`. */
function elementData(page: string, element: string): (name: string) => string {
  const html = readFileSync(new URL(page, fixtures), "utf8");
  const [name, className] = element.split(".");
  const tag = new RegExp(`<${name} class="${className}"[^>]*>`).exec(html)?.[0] ?? "";
  return (name) => {
    const value = new RegExp(` data-${name}="([^"]*)"`).exec(tag)?.[1];
    if (value === undefined) {
      throw new Error(`${page} has no data-${name}`);
    }
    return value
      .replaceAll(/&#(\d+);/g, (_, code: string) => String.fromCodePoint(Number(code)))
      .replaceAll("&quot;", '"')
      .replaceAll("&lt;", "<")
      .replaceAll("&gt;", ">")
      .replaceAll("&amp;", "&");
  };
}

export function pageWords(): SearchWords {
  const data = elementData("pages/expected/stories/read.html", "form.ghg-search");
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
  const data = elementData("pages/expected/search.html", "div.ghg-search-groups");
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
