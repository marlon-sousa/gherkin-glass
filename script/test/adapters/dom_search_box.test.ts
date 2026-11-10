// The search box adapter of src/adapters/dom_search_box.ts, on a page gherkin-glass wrote:
// tests/fixtures/pages/expected/billing/refunds.html, with the service and that site's search.json behind it.
// docs/BEHAVIOUR.md, rules 151 to 156.

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

function open() {
  const window = new Window({ url: "https://example.com/site/billing/refunds.html" });
  window.document.write(readFileSync(new URL("billing/refunds.html", SITE), "utf8"));
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
  return { window, document, field, status, results, type, press, loads: () => loads };
}

test("focus alone fetches nothing and says nothing", () => {
  const page = open();
  page.field.focus();
  page.field.dispatchEvent(new page.window.FocusEvent("focus") as unknown as Event);
  assert.equal(page.loads(), 0);
  assert.equal(page.status.textContent, "");
  assert.equal(page.results.hidden, true);
});

test("results are links relative to the page, their names isolated", async () => {
  const page = open();
  await page.type("saque");
  assert.equal(page.results.hidden, false);
  const links = [...page.results.querySelectorAll("a")];
  assert.equal(links[0]?.getAttribute("href"), "../saque%20em%20conta%20poupan%C3%A7a.html#saque-com-saldo");
  assert.equal(links[0]?.querySelectorAll("bdi").length, 2);
  assert.equal(page.status.textContent, `Results: ${links.length}.`);
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

test("Enter moves to the first result, and is not a submission", async () => {
  const page = open();
  await page.type("refund");
  assert.equal(await page.press("Enter"), true);
  assert.equal(page.document.activeElement, page.results.querySelector("a"));
});

test("an Enter that ends a composition is left alone", async () => {
  const page = open();
  await page.type("refund");
  assert.equal(await page.press("Enter", { isComposing: true }), false);
  assert.equal(await page.press("Enter", { keyCode: 229 }), false);
  assert.notEqual(page.document.activeElement, page.results.querySelector("a"));
});

test("Escape empties the field, the results and the status", async () => {
  const page = open();
  await page.type("refund");
  await page.press("Escape");
  assert.equal(page.field.value, "");
  assert.equal(page.status.textContent, "");
  assert.equal(page.results.hidden, true);
  assert.equal(page.results.children.length, 0);
});
