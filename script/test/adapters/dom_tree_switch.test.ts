// The adapter of src/adapters/dom_tree_switch.ts, on a page gherkin-glass wrote,
// tests/fixtures/pages/expected/billing/refunds.html: docs/BEHAVIOUR.md, rules 87, 88 and 93.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { Window } from "happy-dom";

import { DomTree } from "../../src/adapters/dom_tree.ts";
import { DomTreeSwitch } from "../../src/adapters/dom_tree_switch.ts";
import { NARROW, readTree } from "../../src/policies/tree_state.ts";
import { startFolderTree } from "../../src/services/folder_tree.ts";
import type { Memory } from "../../src/ports/memory.ts";
import type { WindowWidth } from "../../src/ports/width.ts";

const SITE = new URL("../../../tests/fixtures/pages/expected/", import.meta.url);
const DATA = readFileSync(new URL("assets/tree.json", SITE), "utf8");

async function open(narrow: boolean, kept = new Map<string, string>(), data: string | null = DATA): Promise<Window> {
  const window = new Window({ url: "https://example.com/site/billing/refunds.html" });
  window.document.write(readFileSync(new URL("billing/refunds.html", SITE), "utf8"));
  const document = window.document as unknown as Document;
  const memory: Memory = { get: (key) => kept.get(key) ?? null, set: (key, value) => void kept.set(key, value) };
  const source = { load: () => Promise.resolve(data === null ? null : readTree(JSON.parse(data))) };
  const width: WindowWidth = { narrow: () => narrow, onChange: () => {} };
  await startFolderTree(source, new DomTreeSwitch(document), new DomTree(document, new URL("https://example.com/site/"), "billing/refunds.html"), width, memory, {
    sitePath: "/site/",
    page: "billing/refunds.html",
  });
  return window;
}

const checkbox = (window: Window) => window.document.querySelector("#ghg_show_tree") as unknown as HTMLInputElement;
const button = (window: Window) => window.document.querySelector("button.ghg-tree-button") as unknown as HTMLButtonElement;
const landmark = (window: Window) => window.document.querySelector("#ghg_tree")!;

test("the checkbox shows and hides the tree, remembered for the whole site", async () => {
  const kept = new Map<string, string>();
  const window = await open(false, kept);
  assert.equal(checkbox(window).checked, true);
  assert.equal(landmark(window).hasAttribute("hidden"), false);
  checkbox(window).click();
  assert.equal(landmark(window).hasAttribute("hidden"), true);
  assert.equal(kept.get("gherkin-glass:/site/ tree"), "hidden");
  const again = await open(false, kept);
  assert.equal(checkbox(again).checked, false);
  assert.equal(landmark(again).hasAttribute("hidden"), true);
});

test("the button names the tree, says whether it is shown, and shows and hides it", async () => {
  const window = await open(true);
  assert.equal(button(window).textContent, "Folder tree");
  assert.equal(button(window).getAttribute("aria-controls"), "ghg_tree");
  assert.equal(button(window).getAttribute("aria-expanded"), "false");
  assert.equal(landmark(window).hasAttribute("hidden"), true);
  button(window).click();
  assert.equal(button(window).getAttribute("aria-expanded"), "true");
  assert.equal(landmark(window).hasAttribute("hidden"), false);
  button(window).click();
  assert.equal(button(window).getAttribute("aria-expanded"), "false");
  assert.equal(landmark(window).hasAttribute("hidden"), true);
});

test("Escape inside the tree hides it and puts the focus on the button", async () => {
  const window = await open(true);
  button(window).click();
  const item = window.document.querySelector('[role="treeitem"][tabindex="0"]') as unknown as HTMLElement;
  item.focus();
  item.dispatchEvent(new window.KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }) as unknown as Event);
  assert.equal(landmark(window).hasAttribute("hidden"), true);
  assert.equal(button(window).getAttribute("aria-expanded"), "false");
  assert.equal(window.document.activeElement, button(window) as unknown);
});

test("data that cannot be loaded leaves the checkbox and the button disabled and the tree hidden and empty", async () => {
  const window = await open(false, new Map(), null);
  assert.equal(checkbox(window).disabled, true);
  assert.equal(checkbox(window).checked, false);
  assert.equal(button(window).disabled, true);
  assert.equal(landmark(window).hasAttribute("hidden"), true);
  assert.equal(landmark(window).children.length, 0);
});

test("the window the script calls narrow is the one the stylesheet shows the button at", () => {
  const css = readFileSync(new URL("../../../assets/gherkin-glass.css", import.meta.url), "utf8");
  const narrow = css.slice(css.indexOf(`@media ${NARROW} {`));
  assert.ok(narrow.length < css.length, NARROW);
  assert.match(narrow, /\.ghg-option\.ghg-tree-choice \{\s*display: none;/);
  assert.match(narrow, /\.ghg-tree-button \{\s*display: inline-block;/);
});
