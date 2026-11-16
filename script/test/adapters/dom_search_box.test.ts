// The search box adapter of src/adapters/dom_search_box.ts, on a page gherkin-glass wrote:
// tests/fixtures/pages/expected/billing/refunds.html, and the page of search results, search.html, with the
// service and that site's search.json behind it. docs/BEHAVIOUR.md, rules 151 to 156, 214 and 217.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { Window } from "happy-dom";

import { DomSearchBox } from "../../src/adapters/dom_search_box.ts";
import { startSearching } from "../../src/services/searching.ts";
import type { Documents, SearchDocument } from "../../src/ports/documents.ts";
import type { Timer } from "../../src/ports/timer.ts";

const SITE = new URL("../../../tests/fixtures/pages/expected/", import.meta.url);

/** Runs at once, as if the reader had paused. */
const noPause: Timer = { after: (_milliseconds, handler) => handler(), cancel: () => {} };

/** The page at `path` in the site, opened with `query` in its address. */
function open(path = "billing/refunds.html", query = "") {
  const window = new Window({ url: `https://example.com/site/${path}${query}` });
  window.document.write(readFileSync(new URL(path, SITE), "utf8"));
  const found = JSON.parse(readFileSync(new URL("assets/search.json", SITE), "utf8")) as SearchDocument[];
  let loads = 0;
  const documents: Documents = {
    load: () => {
      loads += 1;
      return Promise.resolve(found);
    },
  };
  const document = window.document as unknown as Document;
  const box = DomSearchBox.find(document);
  assert.ok(box);
  startSearching(box, documents, noPause);
  const field = document.querySelector(".ghg-search-field") as HTMLInputElement;
  const status = document.querySelector(".ghg-search-status") as HTMLElement;
  const results = document.querySelector(".ghg-search-results") as HTMLOListElement;
  const link = document.querySelector(".ghg-search-all") as HTMLAnchorElement;
  const went: string[] = [];
  Object.defineProperty(window.location, "assign", { value: (address: string) => went.push(address) });
  const type = async (text: string): Promise<void> => {
    field.value = text;
    field.dispatchEvent(new window.Event("input") as unknown as Event);
    await window.happyDOM.waitUntilComplete();
    await new Promise((resolve) => setTimeout(resolve, 80));
  };
  const press = async (key: string, init: Record<string, unknown> = {}): Promise<boolean> => {
    const event = new window.KeyboardEvent("keydown", { key, cancelable: true, ...init });
    field.dispatchEvent(event as unknown as Event);
    await new Promise((resolve) => setTimeout(resolve, 80));
    return event.defaultPrevented;
  };
  return { window, document, field, status, results, link, went, type, press, loads: () => loads };
}

test("focus alone fetches nothing and says nothing", () => {
  const page = open();
  page.field.focus();
  page.field.dispatchEvent(new page.window.FocusEvent("focus") as unknown as Event);
  assert.equal(page.loads(), 0);
  assert.equal(page.status.textContent, "");
  assert.equal(page.results.hidden, true);
});

test("results are options relative to the page, their names isolated, none chosen", async () => {
  const page = open();
  await page.type("saque");
  assert.equal(page.results.hidden, false);
  assert.equal(page.field.getAttribute("aria-expanded"), "true");
  const options = [...page.results.querySelectorAll("[role=option]")];
  assert.equal(options[0]?.id, "ghg_search_result_0");
  assert.equal(options[0]?.getAttribute("aria-selected"), "false");
  assert.equal(options[0]?.querySelectorAll("bdi").length, 2);
  assert.equal(page.field.hasAttribute("aria-activedescendant"), false);
  assert.equal(page.status.textContent, `Results: ${options.length}.`);
});

test("the arrows choose an option while the focus stays in the field, and are not left to the page", async () => {
  const page = open();
  await page.type("saque");
  page.field.focus();
  assert.equal(await page.press("ArrowDown"), true);
  const first = page.results.querySelector("[role=option]");
  assert.equal(first?.getAttribute("aria-selected"), "true");
  assert.equal(page.field.getAttribute("aria-activedescendant"), first?.id);
  assert.equal(page.document.activeElement, page.field);
  await page.press("ArrowUp");
  assert.equal(page.field.hasAttribute("aria-activedescendant"), false);
  assert.equal(first?.getAttribute("aria-selected"), "false");
});

test("Enter opens the chosen option relative to the page, and is not a submission", async () => {
  const page = open();
  await page.type("saque");
  await page.press("ArrowDown");
  assert.equal(await page.press("Enter"), true);
  assert.deepEqual(page.went, ["../saque%20em%20conta%20poupan%C3%A7a.html#saque-com-saldo"]);
});

test("Enter with no option chosen opens the page of every result", async () => {
  const page = open();
  await page.type("saque com");
  await page.press("Enter");
  assert.deepEqual(page.went, ["../search.html?q=saque+com"]);
});

test("the pointer opens the option it picks", async () => {
  const page = open();
  await page.type("saque");
  const name = page.results.querySelector("bdi") as HTMLElement;
  name.dispatchEvent(new page.window.MouseEvent("click", { bubbles: true }) as unknown as Event);
  assert.deepEqual(page.went, ["../saque%20em%20conta%20poupan%C3%A7a.html#saque-com-saldo"]);
});

test("the link to every result follows the words, and is hidden without them", async () => {
  const page = open();
  assert.equal(page.link.hidden, true);
  await page.type("saque com");
  assert.equal(page.link.hidden, false);
  assert.equal(page.link.textContent, 'All results for "saque com"');
  assert.equal(page.link.querySelector("bdi")?.textContent, "saque com");
  assert.equal(page.link.getAttribute("href"), "../search.html?q=saque+com");
  await page.type(" ");
  assert.equal(page.link.hidden, true);
});

test("the same status is emptied, then read again", async () => {
  const page = open();
  await page.type("saque com");
  const first = page.status.textContent;
  page.field.value = "saque com saldo";
  page.field.dispatchEvent(new page.window.Event("input") as unknown as Event);
  await new Promise((resolve) => setTimeout(resolve, 10));
  assert.equal(page.status.textContent, "");
  await new Promise((resolve) => setTimeout(resolve, 80));
  assert.equal(page.status.textContent, first);
});

test("an Enter that ends a composition is left alone", async () => {
  const page = open();
  await page.type("refund");
  assert.equal(await page.press("Enter", { isComposing: true }), false);
  assert.equal(await page.press("Enter", { keyCode: 229 }), false);
  assert.deepEqual(page.went, []);
});

test("Escape closes the list and keeps the words, and a second empties the field, the results and the status", async () => {
  const page = open();
  await page.type("refund");
  await page.press("Escape");
  assert.equal(page.field.value, "refund");
  assert.equal(page.results.hidden, true);
  assert.equal(page.field.getAttribute("aria-expanded"), "false");
  await page.press("Escape");
  assert.equal(page.field.value, "");
  assert.equal(page.status.textContent, "");
  assert.equal(page.results.hidden, true);
  assert.equal(page.results.children.length, 0);
  assert.equal(page.link.hidden, true);
});

test("the page of search results has the same combobox under its heading, searching as the reader types", async () => {
  const page = open("search.html", "?q=refund");
  const heading = page.document.querySelector("main h1");
  const form = page.document.querySelector("main form.ghg-search");
  assert.ok(heading && form && heading.compareDocumentPosition(form as unknown as Node) & 4);
  await page.type("refunds");
  assert.equal(page.results.hidden, false);
  assert.equal(page.field.getAttribute("aria-expanded"), "true");
  await page.press("ArrowDown");
  const first = page.results.querySelector("[role=option]");
  assert.equal(page.field.getAttribute("aria-activedescendant"), first?.id);
  await page.press("ArrowUp");
  assert.equal(page.field.hasAttribute("aria-activedescendant"), false);
  assert.equal(await page.press("Enter"), true);
  assert.deepEqual(page.went, ["search.html?q=refunds"]);
});
