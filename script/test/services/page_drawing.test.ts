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

// Rule 77: a folder no feature is in has no page.
test("a folder no feature is in draws the page that says nothing is there", async () => {
  const { document } = await drawn({ kind: "folder", folder: "nowhere/" });
  assert.equal(document.querySelector("main h1")!.textContent, "Nothing is at this address");
});
