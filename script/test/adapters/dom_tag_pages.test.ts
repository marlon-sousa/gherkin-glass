// The drawing of src/adapters/dom_tag_pages.tsx, from the views of src/policies/tag_views.ts, against the pages ghg 1 wrote.

import assert from "node:assert/strict";
import { test } from "node:test";

import { tagIndex, tagPage } from "../../src/adapters/dom_tag_pages.tsx";
import { drawn } from "../../src/adapters/jsx/drawing.ts";
import { tagIndexView, tagPageView } from "../../src/policies/tag_views.ts";
import { emptyDocument, markupOf, words, expectedMain } from "../support/drawn_pages.ts";
import { descriptor } from "../support/store_building.ts";
import { tagsDescriptor, tagsFeatures } from "../support/tags_store.ts";

// Rule 102: every tag, with how many scenarios carry it, in the descriptor's order.
test("the index of tags is drawn as ghg 1 wrote it, but for its links' addresses", () => {
  const document = emptyDocument();
  const main = document.querySelector("main")!;
  const view = tagIndexView(words, tagsDescriptor, tagsFeatures);
  main.append(...drawn(document, tagIndex(view)));
  assert.equal(markupOf(main), markupOf(expectedMain("tags/tags/index.html")));
  assert.equal(view.title, "Tags");
});

// Rule 102: with no tag, the list is replaced by a sentence.
test("the index of a site with no tag says so", () => {
  const document = emptyDocument();
  const main = document.querySelector("main")!;
  main.append(...drawn(document, tagIndex(tagIndexView(words, descriptor("Untagged", []), []))));
  assert.equal(markupOf(main), "<h1>Tags</h1><p>No feature file has a tag.</p>");
});

// Rules 103 to 107: each tag's page, its link to another site, its features and the scenarios that carry it.
for (const entry of tagsDescriptor.tags) {
  const name = entry.page.slice("tags/".length);
  test(`the page of ${entry.tag} is drawn as ghg 1 wrote it, but for its links' addresses`, () => {
    const document = emptyDocument();
    const main = document.querySelector("main")!;
    const view = tagPageView(words, tagsDescriptor, tagsFeatures, name)!;
    main.append(...drawn(document, tagPage(view)));
    assert.equal(markupOf(main), markupOf(expectedMain(`tags/tags/${name}.html`)));
    assert.equal(view.title, `Tag ${entry.tag}`);
  });
}

// Rule 100: a name no tag has names no page.
test("a tag page no tag has is not one", () => {
  assert.equal(tagPageView(words, tagsDescriptor, tagsFeatures, "missing"), null);
});
