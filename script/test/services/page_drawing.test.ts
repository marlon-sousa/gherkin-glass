// The service of src/services/page_drawing.ts, with the store in memory, drawing with the real adapters in a document.

import assert from "node:assert/strict";
import { test } from "node:test";

import { DomCanvas } from "../../src/adapters/dom_canvas.ts";
import { DomPages } from "../../src/adapters/dom_pages.ts";
import { ElementBuilder } from "../../src/adapters/element_builder.ts";
import type { PageName } from "../../src/policies/address.ts";
import type { PageFrame } from "../../src/policies/page_frame.ts";
import type { Descriptor } from "../../src/ports/store.ts";
import { pageDrawing } from "../../src/services/page_drawing.ts";
import { billingDescriptor, closedObject } from "../support/billing_store.ts";
import { emptyDocument, words } from "../support/drawn_pages.ts";
import { MemoryStore } from "../support/memory_store.ts";
import { refundsDescriptor, refundsObject } from "../support/refunds_store.ts";
import { tagsDescriptor, tagsFeatures } from "../support/tags_store.ts";

/** Draws `page` of the site `descriptor` describes, in a document whose `main` holds `frontPage`. */
async function drawn(page: PageName, descriptor: Descriptor = refundsDescriptor, frontPage = ""): Promise<{ readonly document: Document; readonly frame: PageFrame }> {
  const document = emptyDocument();
  document.title = "Banking & <more>";
  document.querySelector("main")!.innerHTML = frontPage;
  const make = new ElementBuilder(document);
  const canvas = DomCanvas.of(make)!;
  const draw = pageDrawing({ store: new MemoryStore(descriptor, { refunds: refundsObject }), words, pages: new DomPages(make, canvas, words), canvas });
  const frame = (await draw(page))();
  return { document, frame };
}

// Rules 43 and 85: a feature's page has its title, its language and a breadcrumb through its folders.
test("a feature's page is drawn with its title, its language and the folders above it", async () => {
  const { document, frame } = await drawn({ kind: "feature", page: "billing/refunds" });
  assert.equal(document.querySelector("main h1")!.textContent, "Feature: Refunds & <returns>");
  assert.equal(frame.title, "Feature: Refunds & <returns>, billing/refunds.feature");
  assert.equal(frame.lang, "en");
  assert.deepEqual(
    frame.crumbs.map((crumb) => [crumb.address, crumb.words.map((part) => part.text).join("")]),
    [
      ["#", "Banking & <more>"],
      ["#billing/", "billing"],
      ["#billing/refunds", "Feature: Refunds & <returns>"],
    ],
  );
});

// Rule 239: an address that names no page draws the page that says so.
test("an address that names no page draws the page that says nothing is there", async () => {
  const { document, frame } = await drawn({ kind: "feature", page: "billing/removed" });
  assert.equal(document.querySelector("main h1")!.textContent, "Nothing is at this address");
  assert.equal(document.querySelector("main p")!.textContent, "This site has no page at this address.");
  assert.equal(frame.title, "Nothing is at this address, Banking & <more>");
  assert.deepEqual(frame.crumbs.map((crumb) => crumb.address), ["#"]);
});

// Rule 207: every drawn page of a site of only some feature files starts with the note that says so.
test("a site of only some feature files starts every drawn page with its note", async () => {
  const { document } = await drawn({ kind: "feature", page: "billing/refunds" }, { ...refundsDescriptor, only: true });
  const first = document.querySelector("main")!.firstElementChild!;
  assert.equal(first.getAttribute("role"), "note");
  assert.equal(first.textContent, "This site shows only the feature files given with --only.");
});

// Rule 241: the home page of a shell that arrived holding its front page is that front page, put back.
test("the home page of a shell that holds its front page is the front page, put back", async () => {
  const { document, frame } = await drawn({ kind: "home" }, refundsDescriptor, "<h1>Banking</h1><p>Welcome.</p>");
  assert.equal(document.querySelector("main")!.innerHTML, "<h1>Banking</h1><p>Welcome.</p>");
  assert.equal(frame.title, "Banking & <more>");
});

// Rules 81 and 85: a folder's page, with its title, `en` and a breadcrumb through the folders above it.
test("a folder's page is drawn with its title, its language and the folders above it", async () => {
  const document = emptyDocument();
  const make = new ElementBuilder(document);
  const canvas = DomCanvas.of(make)!;
  const draw = pageDrawing({ store: new MemoryStore(billingDescriptor, { refunds: refundsObject, "closed-1": closedObject }), words, pages: new DomPages(make, canvas, words), canvas });
  const frame = (await draw({ kind: "folder", folder: "billing/archive/" }))();
  assert.equal(document.querySelector("main h1")!.textContent, "archive");
  assert.equal(frame.title, "billing/archive, Banking & <more>");
  assert.equal(frame.lang, "en");
  assert.deepEqual(
    frame.crumbs.map((crumb) => [crumb.address, crumb.words.map((part) => part.text).join("")]),
    [
      ["#", "Banking & <more>"],
      ["#billing/", "billing"],
      ["#billing/archive/", "archive"],
    ],
  );
});

// Rule 77: a version's shell holds no front page, so its home page is drawn as a folder's is.
test("the home page of a shell that holds no front page is drawn as its folder's page", async () => {
  const document = emptyDocument();
  const make = new ElementBuilder(document);
  const canvas = DomCanvas.of(make)!;
  const draw = pageDrawing({ store: new MemoryStore(billingDescriptor, { refunds: refundsObject, "closed-1": closedObject }), words, pages: new DomPages(make, canvas, words), canvas });
  const frame = (await draw({ kind: "home" }))();
  assert.equal(document.querySelector("main h1")!.textContent, "Banking & <more>");
  assert.ok(document.querySelector("main .ghg-totals"));
  assert.equal(frame.title, "Banking & <more>");
  assert.deepEqual(frame.crumbs.map((crumb) => crumb.address), ["#"]);
});

/** A drawing of the pages of tests/fixtures/tags, in `document`. */
function taggedDrawing(document: Document): (page: PageName) => Promise<PageFrame> {
  const make = new ElementBuilder(document);
  const canvas = DomCanvas.of(make)!;
  const objects = Object.fromEntries(tagsFeatures.map((feature) => [feature.entry.object, feature.object]));
  const draw = pageDrawing({ store: new MemoryStore(tagsDescriptor, objects), words, pages: new DomPages(make, canvas, words), canvas });
  return async (page) => (await draw(page))();
}

async function drawnTagged(page: PageName): Promise<{ readonly document: Document; readonly frame: PageFrame }> {
  const document = emptyDocument();
  return { document, frame: await taggedDrawing(document)(page) };
}

const crumbsOf = (frame: PageFrame): string[][] => frame.crumbs.map((crumb) => [crumb.address, crumb.words.map((part) => part.text).join("")]);

// Rule 105: the index of tags and a tag's page, in English, below the home page and the index.
test("the index of tags and a tag's page are drawn with their titles and breadcrumbs", async () => {
  const index = await drawnTagged({ kind: "tags" });
  assert.equal(index.frame.title, "Tags");
  assert.equal(index.frame.lang, "en");
  assert.deepEqual(crumbsOf(index.frame), [
    ["#", "Tagged suite"],
    ["#tags/", "Tags"],
  ]);
  const tagged = await drawnTagged({ kind: "tag", tag: "needs-db" });
  assert.equal(tagged.document.querySelector("main h1")!.textContent, "Tag @needs:db");
  assert.equal(tagged.frame.title, "Tag @needs:db");
  assert.deepEqual(crumbsOf(tagged.frame), [
    ["#", "Tagged suite"],
    ["#tags/", "Tags"],
    ["#tags/needs-db", "Tag @needs:db"],
  ]);
  const missing = await drawnTagged({ kind: "tag", tag: "nowhere" });
  assert.equal(missing.document.querySelector("main h1")!.textContent, "Nothing is at this address");
});

// Rule 143: the page of step sentences, below the home page.
test("the page of step sentences is drawn with its title and breadcrumb", async () => {
  const { frame } = await drawnTagged({ kind: "steps" });
  assert.equal(frame.title, "Step sentences, Tagged suite");
  assert.deepEqual(crumbsOf(frame), [
    ["#", "Tagged suite"],
    ["#steps", "Step sentences"],
  ]);
});

// Rules 216 and 217: the page of search results holds the shell's search box under its `h1`, until another page is drawn.
test("the page of search results holds the search box, which another page puts back", async () => {
  const document = emptyDocument();
  document.querySelector("main")!.before(Object.assign(document.createElement("form"), { className: "ghg-search" }));
  const draw = taggedDrawing(document);
  const frame = await draw({ kind: "search", words: "open  " });
  assert.equal(frame.title, 'Results for "open", Tagged suite');
  assert.deepEqual(crumbsOf(frame), [
    ["#", "Tagged suite"],
    ["#search", "Search results"],
  ]);
  assert.deepEqual(
    [...document.querySelector("main")!.children].map((element) => element.className),
    ["ghg-search-heading", "ghg-search", "ghg-results-status", "ghg-search-groups"],
  );
  await draw({ kind: "steps" });
  assert.equal(document.querySelector("form.ghg-search")!.nextElementSibling, document.querySelector("main"));
  assert.equal((await draw({ kind: "search", words: "" })).title, "Search results, Tagged suite");
});
