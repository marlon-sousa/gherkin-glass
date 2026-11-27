// The policy of src/policies/search_page_view.ts: the view of the page of search results.

import assert from "node:assert/strict";
import { test } from "node:test";

import { searchedHeading } from "../../src/policies/result_groups.ts";
import { searchPageView } from "../../src/policies/search_page_view.ts";
import { words } from "../support/drawn_pages.ts";

// Rule 269: the title the results give ends with the version once, and the view's own title leaves it to the page's frame.
test("the page of search results of a version names the version once, in the title its results give", () => {
  const view = searchPageView(words, "Banking", " free room ", "v1.0.0");
  assert.equal(view.title, 'Results for "free room", Banking');
  assert.equal(searchPageView(words, "Banking", "", "v1.0.0").title, "Search results, Banking");
  assert.equal(searchedHeading(view.words, "cash").title, 'Results for "cash", Banking, v1.0.0');
});
