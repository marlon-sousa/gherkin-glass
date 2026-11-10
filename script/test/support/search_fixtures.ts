// The words a page gives the search box, read from tests/fixtures/pages/expected/stories/read.html,
// and the documents of tests/fixtures/search/search.json.

import { readFileSync } from "node:fs";

import type { SearchDocument } from "../../src/ports/documents.ts";
import type { SearchWords } from "../../src/ports/search_box.ts";

const fixtures = new URL("../../../tests/fixtures/", import.meta.url);

export function pageWords(): SearchWords {
  const html = readFileSync(new URL("pages/expected/stories/read.html", fixtures), "utf8");
  const form = /<form class="ghg-search"[^>]*>/.exec(html)?.[0] ?? "";
  const data = (name: string): string => {
    const value = new RegExp(` data-${name}="([^"]*)"`).exec(form)?.[1];
    if (value === undefined) {
      throw new Error(`the page has no data-${name}`);
    }
    return value.replaceAll("&quot;", '"').replaceAll("&#39;", "'").replaceAll("&lt;", "<").replaceAll("&gt;", ">").replaceAll("&amp;", "&");
  };
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
  };
}

export function documents(): SearchDocument[] {
  return JSON.parse(readFileSync(new URL("search/search.json", fixtures), "utf8")) as SearchDocument[];
}
