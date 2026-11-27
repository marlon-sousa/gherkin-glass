// The service of src/services/page_drawing.ts, drawing the pages of a version, with the store in memory and the real adapters in a document.

import assert from "node:assert/strict";
import { test } from "node:test";

import { DomCanvas } from "../../src/adapters/dom_canvas.ts";
import { DomPages } from "../../src/adapters/dom_pages.ts";
import { ElementBuilder } from "../../src/adapters/element_builder.ts";
import { pageDrawing } from "../../src/services/page_drawing.ts";
import { emptyDocument, words } from "../support/drawn_pages.ts";
import { MemoryStore } from "../support/memory_store.ts";
import { refundsDescriptor, refundsObject } from "../support/refunds_store.ts";

// Rules 259 and 260: a version's feature page has its lists in other versions after its `h1`, and first in each block.
test("a version's feature page has its lists in other versions, which open and close", async () => {
  const document = emptyDocument();
  const make = new ElementBuilder(document);
  const canvas = DomCanvas.of(make)!;
  const versioned = { ...refundsDescriptor, version: "v1.0.0", version_kind: "release" as const };
  const list = { format: 1, development: ["main"], releases: ["v1.0.0"], pre_releases: [], precedence: [["v1.0.0"]] };
  const index = { format: 1, pages: [{ page: "billing/refunds", objects: [{ object: "refunds", versions: ["main", "v1.0.0"] }] }] };
  const draw = pageDrawing({ store: new MemoryStore(versioned, { refunds: refundsObject }, list, index), words, pages: new DomPages(make, canvas, words), canvas });
  (await draw({ kind: "feature", page: "billing/refunds" }))();
  const pageLists = document.querySelector("main h1")!.nextElementSibling!;
  assert.equal(pageLists.className, "ghg-other-versions");
  const button = pageLists.querySelector("button")!;
  assert.equal(button.textContent, "This page in other versions");
  assert.equal(button.getAttribute("aria-expanded"), "false");
  button.click();
  assert.equal(button.getAttribute("aria-expanded"), "true");
  assert.equal(pageLists.querySelector<HTMLElement>(":scope > .ghg-content")!.hidden, false);
  assert.equal(pageLists.querySelector(".ghg-version-group a")!.getAttribute("href"), "../main/index.html#billing/refunds");
  const scenario = document.getElementById("store-credit")!.nextElementSibling!;
  assert.equal(scenario.firstElementChild!.className, "ghg-other-versions");
  assert.equal(scenario.querySelector(".ghg-other-versions button")!.textContent, "In other versions");
});

// Rule 262: without the list of versions, the page is drawn without its lists.
test("a version's feature page whose list of versions cannot be had has no lists", async () => {
  const document = emptyDocument();
  const make = new ElementBuilder(document);
  const canvas = DomCanvas.of(make)!;
  const draw = pageDrawing({ store: new MemoryStore({ ...refundsDescriptor, version: "v1.0.0", version_kind: "release" as const }, { refunds: refundsObject }), words, pages: new DomPages(make, canvas, words), canvas });
  (await draw({ kind: "feature", page: "billing/refunds" }))();
  assert.equal(document.querySelector("main h1")!.textContent, "Feature: Refunds & <returns>");
  assert.equal(document.querySelector(".ghg-other-versions"), null);
});

// Rule 269: the page of search results of a version ends its title with the version once, whatever words are searched for.
test("a version's page of search results names the version once in its title", async () => {
  const document = emptyDocument();
  const make = new ElementBuilder(document);
  const canvas = DomCanvas.of(make)!;
  const draw = pageDrawing({ store: new MemoryStore({ ...refundsDescriptor, version: "v1.0.0", version_kind: "release" as const }, { refunds: refundsObject }), words, pages: new DomPages(make, canvas, words), canvas });
  const titled = async (searched: string): Promise<string> => (await draw({ kind: "search", words: searched }))().title;
  assert.equal(await titled(""), `Search results, ${refundsDescriptor.title}, v1.0.0`);
  assert.equal(await titled("cash"), `Results for "cash", ${refundsDescriptor.title}, v1.0.0`);
});
