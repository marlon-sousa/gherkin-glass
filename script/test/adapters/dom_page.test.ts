// The adapters of src/adapters/, on a page gherkin-glass wrote:
// tests/fixtures/pages/expected/billing/refunds.html, with the service behind them.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { Window } from "happy-dom";

import { DomBackgrounds } from "../../src/adapters/dom_backgrounds.ts";
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
  const document = window.document as unknown as Document;
  const backgrounds = DomBackgrounds.of(document);
  startCollapsing(new DomPage(document), store, window.location.pathname, backgrounds && { backgrounds, siteRoot: "/site/" });
  return window;
}

const button = (window: Window, anchor: string) => window.document.querySelector(`#${anchor} > button`)!;
/** The block of `anchor`: its heading's parent, or the item of an examples block itself. */
const block = (window: Window, anchor: string) => {
  const target = window.document.querySelector(`#${anchor}`)!;
  return target.matches("li") ? target : target.parentElement!;
};
const content = (window: Window, anchor: string) => block(window, anchor).querySelector(":scope > .ghg-content")!;
const expanded = (window: Window, anchor: string): boolean => button(window, anchor).getAttribute("aria-expanded") === "true";

test("every block of the page is found, with the blocks it is inside", () => {
  const window = open();
  const page = new DomPage(window.document as unknown as Document);
  const found = page.blocks().map((block) => [block.anchor, block.parent?.anchor ?? null]);
  assert.deepEqual(found, [
    ["background", null],
    ["outside_rules", null],
    ["refund-a-card-payment", "outside_rules"],
    ["refund-a-card-payment_background_1", "refund-a-card-payment"],
    ["refund-count-items", "outside_rules"],
    ["refund-count-items_background_1", "refund-count-items"],
    ["cheap", "refund-count-items"],
    ["examples", "refund-count-items"],
    ["refund-count-items_scenarios", "refund-count-items"],
    ["refund-count-items-row-1", "refund-count-items_scenarios"],
    ["refunds-after-thirty-days", null],
    ["an-old-order", "refunds-after-thirty-days"],
    ["store-credit", "refunds-after-thirty-days"],
    ["store-credit_background_1", "store-credit"],
    ["store-credit_background_2", "store-credit"],
    ["refund-again-count-times", "refunds-after-thirty-days"],
    ["refund-again-count-times_background_1", "refund-again-count-times"],
    ["refund-again-count-times_background_2", "refund-again-count-times"],
    ["again", "refund-again-count-times"],
    ["refund-again-count-times_scenarios", "refund-again-count-times"],
    ["refund-again-count-times-row-1", "refund-again-count-times_scenarios"],
    ["rule", null],
    ["refund-a-refund", "rule"],
    ["refund-a-refund_background_1", "refund-a-refund"],
  ]);
  // Only a background whose state the checkbox sets says its state while the box is unchecked, and whether it is inside a scenario.
  const following = page.blocks().filter((block) => block.expandedWhenUnchecked !== null);
  assert.deepEqual(
    following.map((block) => [block.anchor, block.expandedWhenUnchecked, block.inScenario]),
    [
      ["background", true, false],
      ["refund-a-card-payment_background_1", false, true],
      ["refund-count-items_background_1", false, true],
      ["an-old-order", true, false],
      ["store-credit_background_1", false, true],
      ["store-credit_background_2", false, true],
      ["refund-again-count-times_background_1", false, true],
      ["refund-again-count-times_background_2", false, true],
      ["refund-a-refund_background_1", false, true],
    ],
  );
});

test("an outline's examples as scenarios start collapsed, and a link to a row's scenario expands them", () => {
  const window = open();
  assert.equal(expanded(window, "refund-again-count-times_scenarios"), false);
  assert.equal(content(window, "refund-again-count-times_scenarios").hasAttribute("hidden"), true);
  (window.document.querySelector(".ghg-collapse-all") as unknown as HTMLElement).click();
  const link = window.document.createElement("a");
  link.setAttribute("href", "#refund-again-count-times-row-1");
  window.document.body.appendChild(link);
  link.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }));
  for (const anchor of ["refunds-after-thirty-days", "refund-again-count-times", "refund-again-count-times_scenarios", "refund-again-count-times-row-1"]) {
    assert.equal(expanded(window, anchor), true, anchor);
  }
});

test("an outline's examples start collapsed, and their buttons and a link to one expand them", () => {
  const window = open();
  for (const anchor of ["cheap", "examples", "again"]) {
    assert.equal(expanded(window, anchor), false, anchor);
    assert.equal(content(window, anchor).hasAttribute("hidden"), true, anchor);
  }
  (button(window, "cheap") as unknown as HTMLElement).click();
  assert.equal(expanded(window, "cheap"), true);
  assert.equal(content(window, "cheap").hasAttribute("hidden"), false);
  (window.document.querySelector(".ghg-collapse-all") as unknown as HTMLElement).click();
  assert.equal(expanded(window, "cheap"), false);
  const link = window.document.createElement("a");
  link.setAttribute("href", "#again");
  window.document.body.appendChild(link);
  link.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }));
  for (const anchor of ["refunds-after-thirty-days", "refund-again-count-times", "again"]) {
    assert.equal(expanded(window, anchor), true, anchor);
  }
});

test("an examples item's state is remembered under its anchor, and wins when the page is opened again", () => {
  const kept = new Map<string, string>();
  const memory: Memory = { get: (key) => kept.get(key) ?? null, set: (key, value) => void kept.set(key, value) };
  const window = open("", memory);
  (button(window, "cheap") as unknown as HTMLElement).click();
  assert.equal(kept.get("gherkin-glass:/site/billing/refunds.html#cheap"), "expanded");
  assert.equal(expanded(open("", memory), "cheap"), true);
});

test("the scenarios outside every rule are one block, which collapses and is found by a link", () => {
  const window = open();
  assert.equal(button(window, "outside_rules").textContent, "Outside any rule");
  (button(window, "outside_rules") as unknown as HTMLElement).click();
  assert.equal(expanded(window, "outside_rules"), false);
  assert.equal(content(window, "outside_rules").hasAttribute("hidden"), true);
  const link = window.document.querySelector('nav a[href="#refund-a-card-payment"]')!;
  link.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }));
  assert.equal(expanded(window, "outside_rules"), true);
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

test("following a link to an anchor of the page by its relative address, as the folder tree's are, expands its block", () => {
  const window = open("#store-credit");
  (window.document.querySelector(".ghg-collapse-all") as unknown as HTMLElement).click();
  const link = window.document.createElement("a");
  link.setAttribute("href", "../billing/refunds.html#store-credit");
  window.document.body.append(link);
  link.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }));
  assert.equal(expanded(window, "store-credit"), true);
  // A link to another page changes nothing here.
  (window.document.querySelector(".ghg-collapse-all") as unknown as HTMLElement).click();
  link.setAttribute("href", "index.html#store-credit");
  link.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }));
  assert.equal(expanded(window, "store-credit"), false);
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

// A folder page: tests/fixtures/pages/expected/billing/index.html, docs/BEHAVIOUR.md, rule 84.

const FOLDER = new URL("../../../tests/fixtures/pages/expected/billing/index.html", import.meta.url);

function openFolder(kept: Map<string, string>): Window {
  const window = new Window({ url: "https://example.com/site/billing/index.html" });
  window.document.write(readFileSync(FOLDER, "utf8"));
  const memory: Memory = { get: (key) => kept.get(key) ?? null, set: (key, value) => void kept.set(key, value) };
  startCollapsing(new DomPage(window.document as unknown as Document), memory, window.location.pathname, null);
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
