// The drawing of src/adapters/dom_results_page.tsx, from the view of src/policies/search_page_view.ts, against the page ghg 1 wrote.

import assert from "node:assert/strict";
import { test } from "node:test";

import { resultsPage } from "../../src/adapters/dom_results_page.tsx";
import { drawn } from "../../src/adapters/jsx/drawing.ts";
import { searchPageView } from "../../src/policies/search_page_view.ts";
import { emptyDocument, markupOf, words, expectedMain } from "../support/drawn_pages.ts";

// Rules 216 and 217: the heading, the search box under it, the page's own status and the words the listing reads.
test("the page of search results is drawn as ghg 1 wrote it, the search box under its heading", () => {
  const written = expectedMain("pages/search.html");
  const document = emptyDocument();
  const main = document.querySelector("main")!;
  const form = document.importNode(written.querySelector("form.ghg-search")!, true);
  main.append(...drawn(document, resultsPage(searchPageView(words, "Banking & <more>", ""), form)));
  assert.equal(markupOf(main), markupOf(written));
});

// Rule 216: words in the address name the page in its title.
test("the page's title names the words searched for", () => {
  assert.equal(searchPageView(words, "Banking", "  free room ").title, 'Results for "free room", Banking');
  assert.equal(searchPageView(words, "Banking", "   ").title, "Search results, Banking");
});
