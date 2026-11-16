// The search page adapter of src/adapters/dom_search_page.ts, on the page gherkin-glass wrote:
// tests/fixtures/pages/expected/search.html, with the service and that site's search.json behind it.
// docs/BEHAVIOUR.md, rules 216 to 220.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { Window } from "happy-dom";

import { DomSearchPage } from "../../src/adapters/dom_search_page.ts";
import { startListing } from "../../src/services/results_listing.ts";
import type { SearchDocument } from "../../src/ports/documents.ts";

const SITE = new URL("../../../tests/fixtures/pages/expected/", import.meta.url);

async function open(query: string) {
  const window = new Window({ url: `https://example.com/site/search.html${query}` });
  window.document.write(readFileSync(new URL("search.html", SITE), "utf8"));
  const found = JSON.parse(readFileSync(new URL("assets/search.json", SITE), "utf8")) as SearchDocument[];
  const document = window.document as unknown as Document;
  const page = DomSearchPage.find(document);
  assert.ok(page);
  await startListing(page, { load: () => Promise.resolve(found) });
  return {
    document,
    heading: document.querySelector("h1") as HTMLElement,
    field: document.querySelector(".ghg-search-field") as HTMLInputElement,
    status: document.querySelector(".ghg-results-status") as HTMLElement,
    groups: document.querySelector(".ghg-search-groups") as HTMLElement,
  };
}

test("a page is found only where the page of search results is", () => {
  const window = new Window({ url: "https://example.com/site/billing/refunds.html" });
  window.document.write(readFileSync(new URL("billing/refunds.html", SITE), "utf8"));
  assert.equal(DomSearchPage.find(window.document as unknown as Document), null);
});

test("the words of the address fill the field, the heading and the title", async () => {
  const page = await open("?q=+refund+full");
  assert.equal(page.field.value, "refund full");
  assert.equal(page.heading.textContent, 'Results for "refund full"');
  assert.equal(page.heading.querySelector("bdi")?.textContent, "refund full");
  assert.equal(page.document.title, 'Results for "refund full", Banking & <more>');
});

test("without words, the page keeps what its HTML says", async () => {
  const page = await open("");
  assert.equal(page.heading.textContent, "Search results");
  assert.equal(page.status.textContent, "");
  assert.equal(page.groups.children.length, 0);
});

test("each feature is a heading linking its page, over a list of its results, their matched words marked", async () => {
  const page = await open("?q=refund");
  const headings = [...page.groups.querySelectorAll("h2")];
  assert.ok(headings.length > 0);
  for (const heading of headings) {
    assert.equal(heading.firstElementChild?.tagName, "A");
    assert.equal(heading.nextElementSibling?.tagName, "UL");
  }
  const result = page.groups.querySelector(".ghg-search-group > li") as HTMLElement;
  const link = result.querySelector("a") as HTMLAnchorElement;
  assert.ok(link.getAttribute("href")?.includes(".html"));
  assert.ok(page.groups.querySelectorAll("mark").length > 0);
  for (const mark of page.groups.querySelectorAll("mark")) {
    assert.match(mark.textContent ?? "", /^refund/i);
  }
  const count = page.groups.querySelectorAll(".ghg-search-group > li").length;
  assert.equal(page.status.textContent, `Results: ${count}. Features: ${headings.length}.`);
});
