// The search page adapter of src/adapters/dom_search_page.ts, on the page of search results of tests/fixtures/pages drawn in
// its shell, with the service and that site's documents, script/test/expected/pages/search.json, behind it.
// docs/BEHAVIOUR.md, rules 216 to 220 and 237.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { DomSearchBox } from "../../src/adapters/dom_search_box.ts";
import { DomSearchPage } from "../../src/adapters/dom_search_page.ts";
import { startSearching } from "../../src/services/searching.ts";
import { startListing } from "../../src/services/results_listing.ts";
import type { SearchDocument } from "../../src/ports/documents.ts";
import { drawnShell } from "../support/drawn_pages.ts";

const FOUND = JSON.parse(readFileSync(new URL("../expected/pages/search.json", import.meta.url), "utf8")) as SearchDocument[];

/** The shell's page of search results, given `words`, the words of its address. */
async function open(words: string) {
  const window = drawnShell("pages/search.html", `https://example.com/site/#search?q=${encodeURIComponent(words)}`);
  const found = FOUND;
  const document = window.document as unknown as Document;
  const page = DomSearchPage.find(document, words);
  assert.ok(page);
  await startListing(page, { load: () => Promise.resolve(found) });
  return {
    window,
    document,
    found,
    heading: document.querySelector("h1") as HTMLElement,
    field: document.querySelector(".ghg-search-field") as HTMLInputElement,
    status: document.querySelector(".ghg-results-status") as HTMLElement,
    groups: document.querySelector(".ghg-search-groups") as HTMLElement,
  };
}

test("a page is found only where the page of search results is", () => {
  const window = drawnShell("pages/billing/refunds.html", "https://example.com/site/#billing/refunds");
  assert.equal(DomSearchPage.find(window.document as unknown as Document, ""), null);
});

test("the words of the address fill the field, the heading and the title", async () => {
  const page = await open("refund full");
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
  const page = await open("refund");
  const headings = [...page.groups.querySelectorAll("h2")];
  assert.ok(headings.length > 0);
  for (const heading of headings) {
    assert.equal(heading.firstElementChild?.tagName, "A");
    assert.equal(heading.nextElementSibling?.tagName, "UL");
  }
  const result = page.groups.querySelector(".ghg-search-group > li") as HTMLElement;
  const link = result.querySelector("a") as HTMLAnchorElement;
  assert.ok(link.getAttribute("href")?.startsWith("#"));
  assert.equal(page.groups.querySelector("h2 a")?.getAttribute("href"), "#billing/refunds");
  assert.ok(page.groups.querySelectorAll("mark").length > 0);
  for (const mark of page.groups.querySelectorAll("mark")) {
    assert.match(mark.textContent ?? "", /^refund/i);
  }
  const count = page.groups.querySelectorAll(".ghg-search-group > li").length;
  assert.equal(page.status.textContent, `Results: ${count}. Features: ${headings.length}.`);
});

test("the search box, started after the page fills its field, links to and goes to the same words", async () => {
  const page = await open("refund");
  const box = DomSearchBox.find(page.document);
  assert.ok(box);
  const went: string[] = [];
  Object.defineProperty(page.window.location, "assign", { value: (address: string) => went.push(address) });
  startSearching(box, { load: () => Promise.resolve(page.found) }, { after: () => {}, cancel: () => {} });
  const link = page.document.querySelector(".ghg-search-all") as HTMLAnchorElement;
  assert.equal(link.hidden, false);
  assert.equal(link.textContent, 'All results for "refund"');
  assert.equal(page.field.getAttribute("aria-expanded"), "false");
  page.field.dispatchEvent(new page.window.KeyboardEvent("keydown", { key: "Enter", cancelable: true }) as unknown as Event);
  assert.deepEqual(went, ["#search?q=refund"]);
});
