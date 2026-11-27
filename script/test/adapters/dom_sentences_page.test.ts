// The drawing of src/adapters/dom_sentences_page.ts, from the view of src/policies/sentences_view.ts, against the page ghg 1 wrote.

import assert from "node:assert/strict";
import { test } from "node:test";

import { sentencesPage } from "../../src/adapters/dom_sentences_page.ts";
import { ElementBuilder } from "../../src/adapters/element_builder.ts";
import { sentencesView } from "../../src/policies/sentences_view.ts";
import { emptyDocument, markupOf, words, writtenMain } from "../support/drawn_pages.ts";
import { sentencesDescriptor, sentencesFeatures } from "../support/sentences_store.ts";
import { descriptor } from "../support/store_building.ts";

// Rules 139 to 144: each sentence, how often it is used, and a link to each use with its values.
test("the page of step sentences is drawn as ghg 1 wrote it, but for its links' addresses", () => {
  const document = emptyDocument();
  const main = document.querySelector("main")!;
  const view = sentencesView(words, sentencesDescriptor, sentencesFeatures);
  main.append(...sentencesPage(new ElementBuilder(document), view));
  assert.equal(markupOf(main), markupOf(writtenMain("step-sentences/expected/steps.html", "steps.html")));
  assert.equal(view.title, "Step sentences, Fixture");
});

// Rule 145: with no step, the page says so.
test("the page of a site with no step says so", () => {
  const document = emptyDocument();
  const main = document.querySelector("main")!;
  main.append(...sentencesPage(new ElementBuilder(document), sentencesView(words, descriptor("Empty", []), [])));
  assert.equal(markupOf(main), "<h1>Step sentences</h1><p>No feature has a step.</p>");
});
