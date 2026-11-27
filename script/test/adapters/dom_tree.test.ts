// The adapter of src/adapters/dom_tree.ts, on the shell of tests/fixtures/pages, with ghg 1's tree of that site, script/test/expected/pages/tree.json: docs/BEHAVIOUR.md, rules 89 to 93.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { Window } from "happy-dom";

import { DomTree } from "../../src/adapters/dom_tree.tsx";
import { DomTreeSwitch } from "../../src/adapters/dom_tree_switch.ts";
import { buildTree, startTreeControls } from "../../src/services/folder_tree.ts";
import type { Memory } from "../../src/ports/memory.ts";
import type { TreeItem } from "../../src/ports/tree.ts";
import { SHELL } from "../support/drawn_pages.ts";

const ITEMS = JSON.parse(readFileSync(new URL("../expected/pages/tree.json", import.meta.url), "utf8")) as TreeItem[];

function open(kept = new Map<string, string>()): Window {
  const window = new Window({ url: "https://example.com/site/#billing/refunds" });
  window.document.write(SHELL);
  const document = window.document as unknown as Document;
  const memory: Memory = { get: (key) => kept.get(key) ?? null, set: (key, value) => void kept.set(key, value) };
  const wide = { narrow: () => false, onChange: () => {} };
  startTreeControls(new DomTreeSwitch(document), wide, memory, "/site/");
  buildTree(new DomTree(document), memory, "/site/", ITEMS, ["#billing/refunds", "#billing/"]);
  return window;
}

const items = (window: Window) => [...window.document.querySelectorAll('[role="treeitem"]')] as unknown as HTMLAnchorElement[];
const item = (window: Window, text: string) => items(window).find((link) => link.textContent === text)!;
const visible = (window: Window) => items(window).filter((link) => !link.closest("[hidden]")).map((link) => link.textContent);
const focused = (window: Window) => window.document.activeElement?.textContent;
const press = (window: Window, key: string) =>
  window.document.activeElement!.dispatchEvent(new window.KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }));

test("the tree is built in its landmark, named as it, with every item a link to its page in the shell", () => {
  const window = open();
  const tree = window.document.querySelector('nav.ghg-tree > ul[role="tree"]')!;
  assert.equal(tree.getAttribute("aria-label"), "Folder tree");
  const home = items(window)[0]!;
  assert.equal(home.textContent, "Banking & <more>");
  assert.equal(home.getAttribute("href"), "#");
  assert.equal(item(window, "Scenario: Store credit").getAttribute("href"), "#billing/refunds#store-credit");
  // Feature text inside gherkin-glass's words is isolated in a bdi.
  const isolated = [...item(window, "Scenario: Store credit").querySelectorAll("bdi")].map((bdi) => bdi.textContent);
  assert.deepEqual(isolated, ["Store credit"]);
  assert.equal(item(window, "Feature: Closed accounts").getAttribute("href"), "#billing/archive/closed%20%231");
  // A feature's feature text, and that of the items inside it, has its page's lang; the tree's own words have none.
  const languages = (text: string) => [...item(window, text).querySelectorAll("bdi")].map((bdi) => bdi.getAttribute("lang"));
  assert.deepEqual(languages("Feature: Saque em conta poupança"), ["pt"]);
  assert.deepEqual(languages("Scenario: Saque com saldo"), ["pt"]);
  assert.ok(item(window, "Scenario: Saque com saldo").closest("nav [lang]") === null);
  assert.ok(home.closest("nav [lang]") === null);
});

test("the current page is marked, and it and the items it is inside are expanded, everything else collapsed", () => {
  const window = open();
  const current = items(window).filter((link) => link.getAttribute("aria-current") === "page");
  assert.deepEqual(current.map((link) => link.textContent), ["Feature: Refunds & <returns>"]);
  assert.equal(item(window, "billing").getAttribute("aria-expanded"), "true");
  assert.equal(item(window, "Feature: Refunds & <returns>").getAttribute("aria-expanded"), "true");
  assert.equal(item(window, "archive").getAttribute("aria-expanded"), "false");
  assert.equal(item(window, "reports").getAttribute("aria-expanded"), "false");
  assert.equal(item(window, "Home") ?? null, null);
  // A collapsed item's group is hidden, and an item with nothing inside it has no marker.
  assert.equal(visible(window).includes("Feature: Closed accounts"), false);
  assert.equal(items(window)[0]!.querySelector(".ghg-marker"), null);
  const owned = item(window, "billing").getAttribute("aria-owns")!;
  assert.equal(window.document.getElementById(owned)!.getAttribute("role"), "group");
  assert.match(owned, /_/);
});

test("the tree is one Tab stop, on the current page's item", () => {
  const window = open();
  const stops = items(window).filter((link) => link.tabIndex === 0);
  assert.deepEqual(stops.map((link) => link.textContent), ["Feature: Refunds & <returns>"]);
});

test("the arrow keys, Home and End move through the visible items and expand and collapse them", () => {
  const kept = new Map<string, string>();
  const window = open(kept);
  const billing = item(window, "billing");
  billing.focus();
  press(window, "ArrowUp");
  assert.equal(focused(window), "Banking & <more>");
  press(window, "ArrowUp");
  assert.equal(focused(window), "Banking & <more>");
  press(window, "ArrowDown");
  press(window, "ArrowDown");
  assert.equal(focused(window), "archive");
  press(window, "ArrowRight");
  assert.equal(item(window, "archive").getAttribute("aria-expanded"), "true");
  assert.equal(kept.get("gherkin-glass:/site/ tree #billing/archive/"), "expanded");
  assert.equal(focused(window), "archive");
  press(window, "ArrowRight");
  assert.equal(focused(window), "Feature: Closed accounts");
  press(window, "ArrowLeft");
  assert.equal(focused(window), "archive");
  press(window, "ArrowLeft");
  assert.equal(item(window, "archive").getAttribute("aria-expanded"), "false");
  assert.equal(kept.get("gherkin-glass:/site/ tree #billing/archive/"), "collapsed");
  press(window, "ArrowLeft");
  assert.equal(focused(window), "billing");
  press(window, "End");
  assert.equal(focused(window), visible(window).at(-1));
  press(window, "Home");
  assert.equal(focused(window), "Banking & <more>");
  // The focused item is the one Tab stop.
  assert.deepEqual(items(window).filter((link) => link.tabIndex === 0).map((link) => link.textContent), ["Banking & <more>"]);
});

test("a letter moves to the next visible item starting with it, and * expands the items beside", () => {
  const window = open();
  items(window)[0]!.focus();
  press(window, "R");
  assert.equal(focused(window), "Rule: Refunds after thirty days");
  press(window, "r");
  assert.equal(focused(window), "Rule");
  press(window, "r");
  assert.equal(focused(window), "reports");
  // From the last match, the search starts again from the first item.
  press(window, "b");
  assert.equal(focused(window), "Banking & <more>");
  press(window, "b");
  assert.equal(focused(window), "billing");
  press(window, "*");
  for (const sibling of ["billing", "reports", "stories"]) {
    assert.equal(item(window, sibling).getAttribute("aria-expanded"), "true", sibling);
  }
  assert.equal(item(window, "archive").getAttribute("aria-expanded"), "false");
});

test("pressing a marker expands or collapses its item, and Space follows the link", () => {
  const window = open();
  const reports = item(window, "reports");
  const followed: string[] = [];
  window.document.addEventListener("click", (event) => {
    if (!event.defaultPrevented) {
      followed.push((event.target as unknown as Element).closest("a")!.textContent ?? "");
      event.preventDefault();
    }
  });
  (reports.querySelector(".ghg-marker") as unknown as HTMLElement).click();
  assert.equal(reports.getAttribute("aria-expanded"), "true");
  assert.deepEqual(followed, []);
  reports.focus();
  press(window, " ");
  assert.deepEqual(followed, ["reports"]);
});

test("without a current page, the first item is the Tab stop", () => {
  const window = new Window({ url: "https://example.com/site/#tags/" });
  window.document.write(SHELL);
  new DomTree(window.document as unknown as Document).build(ITEMS, null, () => false);
  assert.equal(items(window)[0]!.tabIndex, 0);
});
