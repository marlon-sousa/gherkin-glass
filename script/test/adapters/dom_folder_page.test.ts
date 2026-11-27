// The drawing of src/adapters/dom_folder_page.tsx, from the views of src/policies/folder_view.ts, against the pages ghg 1 wrote.

import assert from "node:assert/strict";
import { test } from "node:test";

import { folderPage } from "../../src/adapters/dom_folder_page.tsx";
import { drawn } from "../../src/adapters/jsx/drawing.ts";
import { folderView } from "../../src/policies/folder_view.ts";
import { billingDescriptor, billingFeatures } from "../support/billing_store.ts";
import { emptyDocument, markupOf, words, expectedMain } from "../support/drawn_pages.ts";
import { block, descriptor, entry, object } from "../support/store_building.ts";

// Rules 81, 83 and 84: the folder's name, its subfolders and its features, each with its count and its contents.
test("a folder's page is drawn as ghg 1 wrote it, but for its links' addresses", () => {
  const document = emptyDocument();
  const main = document.querySelector("main")!;
  const view = folderView(words, billingDescriptor, billingFeatures, "billing/")!;
  main.append(...drawn(document, folderPage(view)));
  assert.equal(markupOf(main), markupOf(expectedMain("pages/billing/index.html")));
  assert.equal(view.title, "billing, Banking & <more>");
});

// Rule 84: a feature without contents has its count as text, with no button.
test("a feature with no rule and no scenario has its count as text", () => {
  const document = emptyDocument();
  const main = document.querySelector("main")!;
  const view = folderView(words, billingDescriptor, billingFeatures, "billing/archive/")!;
  main.append(...drawn(document, folderPage(view)));
  assert.equal(markupOf(main), markupOf(expectedMain("pages/billing/archive/index.html")));
  assert.equal(view.title, "billing/archive, Banking & <more>");
});

// Rules 82, 105 and 146: the home page's totals, then its links to the tags' index and to the step sentences.
test("the home page has the site's totals and its links to the tags and the step sentences", () => {
  const document = emptyDocument();
  const main = document.querySelector("main")!;
  main.append(...drawn(document, folderPage(folderView(words, billingDescriptor, billingFeatures, "")!)));
  assert.deepEqual(
    [...main.querySelectorAll(".ghg-totals li")].map((total) => total.textContent),
    ["2 folders", "2 features", "3 rules", "5 scenarios"],
  );
  assert.deepEqual(
    [...main.querySelectorAll("p a")].map((link) => [link.getAttribute("href"), link.textContent]),
    [
      ["#tags/", "Tags"],
      ["#steps", "Step sentences"],
    ],
  );
  assert.equal(main.querySelector("h1")?.textContent, "Banking & <more>");
  assert.deepEqual(
    [...main.querySelectorAll(".ghg-folders a")].map((link) => link.getAttribute("href")),
    ["#billing/"],
  );
  assert.equal(main.querySelector(".ghg-features"), null);
});

// Rule 208: a site of a list that names no feature file says so on its home page.
test("the home page of a list that names no feature file says so", () => {
  const view = folderView(words, descriptor("Empty", [], { only: true }), [], "")!;
  assert.equal(view.nothingGiven, "No feature file was given with --only.");
  assert.deepEqual(view.totals, ["0 folders", "0 features", "0 rules", "0 scenarios"]);
  assert.equal(view.folders, null);
  assert.equal(view.features, null);
});

// Rule 77: a folder with no feature file at any depth below it has no page.
test("a folder no feature is in has no page", () => {
  const feature = { entry: entry("a/b/c.feature", "c"), object: object(block("feature", "Feature", "C", "")) };
  const site = descriptor("Site", [feature.entry]);
  assert.equal(folderView(words, site, [feature], "b/"), null);
  assert.equal(folderView(words, site, [feature], "a/")?.folders?.links[0]?.address, "#a/b/");
  assert.equal(folderView(words, site, [feature], "a/b/")?.features?.entries[0]?.address, "#a/b/c");
});
