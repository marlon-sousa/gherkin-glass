// The adapters of src/adapters/dom_canvas.ts and src/adapters/dom_address.ts, on a shell in a document.

import assert from "node:assert/strict";
import { test } from "node:test";

import { Window } from "happy-dom";

import { DomAddress } from "../../src/adapters/dom_address.ts";
import { DomCanvas } from "../../src/adapters/dom_canvas.ts";
import { ElementBuilder } from "../../src/adapters/element_builder.ts";

const SHELL = `<!doctype html><html lang="en"><head><title>Site</title></head><body>
<a class="ghg-skip" href="##ghg_main">Skip to content</a>
<nav class="ghg-breadcrumb" aria-label="Breadcrumb"></nav>
<main id="ghg_main" tabindex="-1" aria-busy="true"></main>
<p id="ghg_announcement" class="ghg-announcement" role="status"></p>
</body></html>`;

function shell(): { readonly window: Window; readonly canvas: DomCanvas } {
  const window = new Window({ url: "https://example.com/site/v1/index.html#billing/" });
  window.document.write(SHELL);
  return { window, canvas: DomCanvas.of(new ElementBuilder(window.document as unknown as Document))! };
}

// Rule 240: the main is busy while drawn, and the title is said in the live region.
test("the main is marked busy while drawn, and words are said in the live region", () => {
  const { window, canvas } = shell();
  canvas.setBusy(false);
  assert.equal(window.document.querySelector("main")!.getAttribute("aria-busy"), null);
  canvas.setBusy(true);
  assert.equal(window.document.querySelector("main")!.getAttribute("aria-busy"), "true");
  canvas.announce("Tags, Site");
  assert.equal(window.document.querySelector("#ghg_announcement")!.textContent, "Tags, Site");
});

// Rules 85 and 42: the breadcrumb ends with the page's own crumb, and the page's language is the document's.
test("the frame sets the document's language and the breadcrumb, its last crumb the current page", () => {
  const { window, canvas } = shell();
  canvas.setFrame({ title: "", lang: "pt", crumbs: [{ address: "#", words: [{ text: "Site", isolated: false }], lang: null }, { address: "#saque", words: [{ text: "Saque", isolated: true }], lang: "pt" }] });
  assert.equal(window.document.documentElement.lang, "pt");
  assert.equal(
    window.document.querySelector("nav.ghg-breadcrumb")!.innerHTML,
    '<ol><li><a href="#">Site</a></li><li><a href="#saque" aria-current="page"><bdi lang="pt">Saque</bdi></a></li></ol>',
  );
});

// Rule 240: the focus moves to the h1 or to an element, which takes it without being a stop of the Tab key.
test("the focus moves to the h1, or to the element an id names, which then holds tabindex -1", () => {
  const { window, canvas } = shell();
  window.document.querySelector("main")!.innerHTML = '<h1>Title</h1><h2 id="store-credit">Store credit</h2>';
  canvas.focusHeading();
  assert.equal(window.document.activeElement!.tagName, "H1");
  assert.equal(canvas.focusElement("store-credit"), true);
  assert.equal(window.document.activeElement!.id, "store-credit");
  assert.equal(window.document.activeElement!.getAttribute("tabindex"), "-1");
  assert.equal(canvas.focusElement("nowhere"), false);
});

// Rule 236: a link to an element of the page shown is caught, leaving the address as it was.
test("a link written ## is caught, and the address stays as it was", () => {
  const { window } = shell();
  const followed: string[] = [];
  new DomAddress(window as unknown as globalThis.Window).onChange((hash) => followed.push(hash));
  (window.document.querySelector("a.ghg-skip") as unknown as HTMLElement).click();
  assert.deepEqual(followed, ["##ghg_main"]);
  assert.equal(window.location.hash, "#billing/");
});
