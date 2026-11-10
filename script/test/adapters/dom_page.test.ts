// The adapters of src/adapters/, on a page gherkin-glass wrote:
// tests/fixtures/pages/expected/billing/refunds.html, with the service behind them.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { Window } from "happy-dom";

import { DomPage } from "../../src/adapters/dom_page.ts";
import { LocalMemory } from "../../src/adapters/local_memory.ts";
import { startCollapsing } from "../../src/services/collapsing.ts";
import type { Memory } from "../../src/ports/memory.ts";

const PAGE = new URL("../../../tests/fixtures/pages/expected/billing/refunds.html", import.meta.url);

/** The page opened at `hash`, its script started with `memory`. */
function open(hash = "", memory: Memory = new Map() as unknown as Memory): Window {
  const window = new Window({ url: `https://example.com/site/billing/refunds.html${hash}` });
  window.document.write(readFileSync(PAGE, "utf8"));
  const kept = new Map<string, string>();
  const store: Memory = memory instanceof Map ? { get: (key) => kept.get(key) ?? null, set: (key, value) => void kept.set(key, value) } : memory;
  startCollapsing(new DomPage(window.document as unknown as Document), store, window.location.pathname);
  return window;
}

const button = (window: Window, anchor: string) => window.document.querySelector(`#${anchor} > button`)!;
const content = (window: Window, anchor: string) => window.document.querySelector(`#${anchor}`)!.parentElement!.querySelector(":scope > .ghg-content")!;
const expanded = (window: Window, anchor: string): boolean => button(window, anchor).getAttribute("aria-expanded") === "true";

test("every block of the page is found, with the blocks it is inside", () => {
  const window = open();
  const page = new DomPage(window.document as unknown as Document);
  const found = page.blocks().map((block) => [block.anchor, block.parent?.anchor ?? null]);
  assert.deepEqual(found, [
    ["background", null],
    ["refund-a-card-payment", null],
    ["refund-count-items", null],
    ["cheap", "refund-count-items"],
    ["examples", "refund-count-items"],
    ["refunds-after-thirty-days", null],
    ["an-old-order", "refunds-after-thirty-days"],
    ["store-credit", "refunds-after-thirty-days"],
    ["refund-again-count-times", "refunds-after-thirty-days"],
    ["again", "refund-again-count-times"],
    ["rule", null],
    ["refund-a-refund", "rule"],
  ]);
});

test("pressing a button hides its block's content and says so, and pressing it again shows it", () => {
  const window = open();
  (button(window, "refunds-after-thirty-days") as unknown as HTMLElement).click();
  assert.equal(expanded(window, "refunds-after-thirty-days"), false);
  assert.equal(content(window, "refunds-after-thirty-days").hasAttribute("hidden"), true);
  // The blocks inside keep their own state, hidden with the rule's content.
  assert.equal(expanded(window, "store-credit"), true);
  (button(window, "refunds-after-thirty-days") as unknown as HTMLElement).click();
  assert.equal(content(window, "refunds-after-thirty-days").hasAttribute("hidden"), false);
});

test("collapse all and expand all set every block of the page", () => {
  const window = open();
  (window.document.querySelector(".ghg-collapse-all") as unknown as HTMLElement).click();
  assert.equal(window.document.querySelectorAll(".ghg-content:not([hidden])").length, 0);
  (window.document.querySelector(".ghg-expand-all") as unknown as HTMLElement).click();
  assert.equal(window.document.querySelectorAll(".ghg-content[hidden]").length, 0);
});

test("opening the page at an anchor, encoded or not, expands it and the blocks around it", () => {
  const kept = new Map<string, string>();
  for (const anchor of ["refunds-after-thirty-days", "refund-again-count-times", "again"]) {
    kept.set(`gherkin-glass:/site/billing/refunds.html#${anchor}`, "collapsed");
  }
  const memory: Memory = { get: (key) => kept.get(key) ?? null, set: (key, value) => void kept.set(key, value) };
  const window = open("#%61gain", memory);
  for (const anchor of ["refunds-after-thirty-days", "refund-again-count-times", "again"]) {
    assert.equal(expanded(window, anchor), true, anchor);
  }
  assert.equal(kept.get("gherkin-glass:/site/billing/refunds.html#again"), "expanded");
});

test("following a contents link expands its block, but a click that opens another tab does not", () => {
  const window = open();
  (window.document.querySelector(".ghg-collapse-all") as unknown as HTMLElement).click();
  const link = window.document.querySelector('nav a[href="#store-credit"]')!;
  link.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true, ctrlKey: true }));
  assert.equal(expanded(window, "store-credit"), false);
  link.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }));
  assert.equal(expanded(window, "store-credit"), true);
  assert.equal(expanded(window, "refunds-after-thirty-days"), true);
});

test("the browser's storage keeps each state, and a storage that refuses keeps nothing without failing", () => {
  const window = new Window({ url: "https://example.com/" });
  const global = globalThis as unknown as { window?: unknown };
  global.window = window;
  try {
    const memory = new LocalMemory();
    memory.set("k", "collapsed");
    assert.equal(memory.get("k"), "collapsed");
    Object.defineProperty(window, "localStorage", {
      get() {
        throw new Error("storage is forbidden");
      },
    });
    memory.set("k", "expanded");
    assert.equal(memory.get("k"), null);
  } finally {
    delete global.window;
  }
});

// A folder page: tests/fixtures/pages/expected/billing/index.html, docs/BEHAVIOUR.md, rule 63.

const FOLDER = new URL("../../../tests/fixtures/pages/expected/billing/index.html", import.meta.url);

function openFolder(kept: Map<string, string>): Window {
  const window = new Window({ url: "https://example.com/site/billing/index.html" });
  window.document.write(readFileSync(FOLDER, "utf8"));
  const memory: Memory = { get: (key) => kept.get(key) ?? null, set: (key, value) => void kept.set(key, value) };
  startCollapsing(new DomPage(window.document as unknown as Document), memory, window.location.pathname);
  return window;
}

test("a folder page's feature shows its contents by its button, remembered under its link's address", () => {
  const kept = new Map<string, string>();
  const window = openFolder(kept);
  const page = new DomPage(window.document as unknown as Document);
  assert.deepEqual(
    page.blocks().map((block) => [block.anchor, block.parent]),
    [["refunds.html", null]],
  );
  const toggle = window.document.querySelector(".ghg-feature > button") as unknown as HTMLElement;
  const contents = window.document.querySelector(".ghg-feature > .ghg-content")!;
  assert.equal(toggle.getAttribute("aria-expanded"), "false");
  assert.equal(contents.hasAttribute("hidden"), true);
  toggle.click();
  assert.equal(toggle.getAttribute("aria-expanded"), "true");
  assert.equal(contents.hasAttribute("hidden"), false);
  assert.equal(kept.get("gherkin-glass:/site/billing/index.html#refunds.html"), "expanded");
  const again = openFolder(kept);
  assert.equal(again.document.querySelector(".ghg-feature > button")!.getAttribute("aria-expanded"), "true");
});
