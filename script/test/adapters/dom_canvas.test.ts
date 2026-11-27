// The adapter of src/adapters/dom_canvas.ts, on a shell in a document.

import assert from "node:assert/strict";
import { test } from "node:test";

import { Window } from "happy-dom";

import { DomCanvas } from "../../src/adapters/dom_canvas.ts";
import { ElementBuilder } from "../../src/adapters/element_builder.ts";

const SHELL = `<!doctype html><html lang="en"><head><title>Site</title></head><body>
<a class="ghg-skip" href="##ghg_main">Skip to content</a>
<nav class="ghg-breadcrumb" aria-label="Breadcrumb"></nav>
<main id="ghg_main" tabindex="-1" aria-busy="true"></main>
<p id="ghg_announcement" class="ghg-announcement" role="status"></p>
</body></html>`;

function shell(markup = SHELL): { readonly window: Window; readonly canvas: DomCanvas } {
  const window = new Window({ url: "https://example.com/site/v1/index.html#billing/" });
  window.document.write(markup);
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

const FRONT_PAGE_SHELL = SHELL.replace('<main id="ghg_main" tabindex="-1" aria-busy="true"></main>', '<form class="ghg-search"></form><main id="ghg_main" tabindex="-1" aria-busy="true"><h1>Banking</h1><button>Open</button></main>');

// Rule 241: the front page is put back as it arrived, whatever a visit added to it.
test("the front page is put back as it arrived, each time a fresh copy", () => {
  const { window, canvas } = shell(FRONT_PAGE_SHELL);
  const main = window.document.querySelector("main")!;
  main.querySelector("button")!.setAttribute("aria-expanded", "true");
  canvas.draw([]);
  assert.equal(canvas.restoreFrontPage(), "Site");
  assert.equal(main.innerHTML, "<h1>Banking</h1><button>Open</button>");
  const first = main.querySelector("button");
  canvas.restoreFrontPage();
  assert.notEqual(main.querySelector("button"), first);
});

// Rules 216 and 217: the search box drawn in the page of search results goes back to the shell with any other page.
test("the search box goes back to its place in the shell when another page or the front page is drawn", () => {
  const { window, canvas } = shell(FRONT_PAGE_SHELL);
  const form = canvas.searchForm()!;
  const main = window.document.querySelector("main")!;
  canvas.draw([form]);
  assert.equal(form.parentElement, main);
  canvas.restoreFrontPage();
  assert.equal(form.nextElementSibling, main);
  canvas.draw([form]);
  canvas.drawFailure("This page could not be loaded.");
  assert.equal(form.nextElementSibling, main);
});
