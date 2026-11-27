// The drawing of src/adapters/dom_feature_page.ts, from the views of src/policies/feature_view.ts, against the pages ghg 1 wrote.

import assert from "node:assert/strict";
import { test } from "node:test";

import { featurePage } from "../../src/adapters/dom_feature_page.tsx";
import { drawn } from "../../src/adapters/jsx/drawing.ts";
import { featureView } from "../../src/policies/feature_view.ts";
import { emptyDocument, markupOf, words, expectedMain } from "../support/drawn_pages.ts";
import { backgroundsDescriptor, backgroundsEntries, bothObject, noStepsObject, regraObject, uncheckedDescriptor } from "../support/backgrounds_store.ts";
import { outlinesDescriptor, outlinesEntry, outlinesObject } from "../support/outlines_store.ts";
import { refundsDescriptor, refundsEntry, refundsObject } from "../support/refunds_store.ts";

// Rule 40: the page is drawn in the elements ghg 1 wrote, rules 43 to 51 and 129 to 132 among them.
test("an outline's page is drawn as ghg 1 wrote it, but for its links' addresses", () => {
  const document = emptyDocument();
  const main = document.querySelector("main")!;
  const view = featureView(words, outlinesDescriptor, outlinesEntry, outlinesObject);
  main.append(...drawn(document, featurePage(view, words.text("page-contents"), () => null, null)));
  assert.equal(markupOf(main), markupOf(expectedMain("pages/outlines.html")));
  assert.equal(view.title, "Feature: Withdraw cash at the counter, outlines.feature");
  assert.equal(view.lang, "en");
});

// Rules 43 to 51 and 94 to 98: tags, descriptions, backgrounds inside scenarios, rules, the group outside them and the note of a keyword.
test("a page with rules, backgrounds, tags and a note is drawn as ghg 1 wrote it, but for its links' addresses", () => {
  const document = emptyDocument();
  const main = document.querySelector("main")!;
  const view = featureView(words, refundsDescriptor, refundsEntry, refundsObject);
  main.append(...drawn(document, featurePage(view, words.text("page-contents"), () => null, null)));
  assert.equal(markupOf(main), markupOf(expectedMain("pages/billing/refunds.html")));
});

// Rule 51: the starting states are the descriptor's, here with rules and scenarios collapsed and no source link.
test("a page is drawn in the starting states its descriptor gives, as ghg 1 wrote it", () => {
  const document = emptyDocument();
  const main = document.querySelector("main")!;
  const descriptor = { ...refundsDescriptor, title: "Feature files", start: { ...refundsDescriptor.start, collapse_rules: true, collapse_scenarios: true, show_tree: false } };
  const view = featureView(words, descriptor, { ...refundsEntry, source: null }, refundsObject);
  main.append(...drawn(document, featurePage(view, words.text("page-contents"), () => null, null)));
  assert.equal(markupOf(main), markupOf(expectedMain("pages/collapsed/billing/refunds.html")));
});

// Rule 45: a doc string's content is the element's text, its first line feed included.
test("a doc string keeps its content exactly, a first empty line included", () => {
  const document = emptyDocument();
  const main = document.querySelector("main")!;
  main.append(...drawn(document, featurePage(featureView(words, refundsDescriptor, refundsEntry, refundsObject), "Contents", () => null, null)));
  assert.equal(main.querySelector("pre.ghg-doc-string")!.textContent, "\nRefunded in full.\n  Indented.");
});

// Rules 94 to 98: the backgrounds a scenario starts with, in their states for each state of the checkbox, a file in Portuguese with notes, and a checkbox that cannot act.
test("pages with backgrounds are drawn as ghg 1 wrote them, with the checkbox checked and unchecked", () => {
  const cases = [
    { descriptor: backgroundsDescriptor, folder: "" },
    { descriptor: uncheckedDescriptor, folder: "unchecked/" },
  ];
  const objects = { both: bothObject, regra: regraObject, noSteps: noStepsObject };
  for (const { descriptor, folder } of cases) {
    for (const [key, entry] of Object.entries(backgroundsEntries)) {
      const document = emptyDocument();
      const main = document.querySelector("main")!;
      const view = featureView(words, descriptor, entry, objects[key as keyof typeof objects]);
      main.append(...drawn(document, featurePage(view, words.text("page-contents"), () => null, null)));
      assert.equal(markupOf(main), markupOf(expectedMain(`backgrounds/${folder}${entry.page}.html`)), `${folder}/${entry.page}`);
    }
  }
});
