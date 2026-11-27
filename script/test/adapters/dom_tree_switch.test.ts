// The adapter of src/adapters/dom_tree_switch.ts, on the shell of tests/fixtures/pages: docs/BEHAVIOUR.md, rules 87, 88 and 93.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { Window } from "happy-dom";

import { DomTree } from "../../src/adapters/dom_tree.ts";
import { DomTreeSwitch } from "../../src/adapters/dom_tree_switch.ts";
import { NARROW } from "../../src/policies/tree_state.ts";
import { buildTree, startTreeControls } from "../../src/services/folder_tree.ts";
import type { Memory } from "../../src/ports/memory.ts";
import type { TreeItem } from "../../src/ports/tree.ts";
import type { WindowWidth } from "../../src/ports/width.ts";
import { SHELL } from "../support/drawn_pages.ts";

const ITEMS = JSON.parse(readFileSync(new URL("../expected/pages/tree.json", import.meta.url), "utf8")) as TreeItem[];

/** The shell with its tree of `items`, or with its controls disabled when there are none. */
function open(narrow: boolean, kept = new Map<string, string>(), items: readonly TreeItem[] | null = ITEMS): Window {
  const window = new Window({ url: "https://example.com/site/#billing/refunds" });
  window.document.write(SHELL);
  const document = window.document as unknown as Document;
  const memory: Memory = { get: (key) => kept.get(key) ?? null, set: (key, value) => void kept.set(key, value) };
  const width: WindowWidth = { narrow: () => narrow, onChange: () => {} };
  const controls = startTreeControls(new DomTreeSwitch(document), width, memory, "/site/");
  if (items === null) {
    controls.disable();
  } else {
    buildTree(new DomTree(document), memory, "/site/", items, ["#billing/refunds", "#billing/"]);
  }
  return window;
}

const checkbox = (window: Window) => window.document.querySelector("#ghg_show_tree") as unknown as HTMLInputElement;
const button = (window: Window) => window.document.querySelector("button.ghg-tree-button") as unknown as HTMLButtonElement;
const landmark = (window: Window) => window.document.querySelector("#ghg_tree")!;

test("the checkbox shows and hides the tree, remembered for the whole site", () => {
  const kept = new Map<string, string>();
  const window = open(false, kept);
  assert.equal(checkbox(window).checked, true);
  assert.equal(landmark(window).hasAttribute("hidden"), false);
  checkbox(window).click();
  assert.equal(landmark(window).hasAttribute("hidden"), true);
  assert.equal(kept.get("gherkin-glass:/site/ tree"), "hidden");
  const again = open(false, kept);
  assert.equal(checkbox(again).checked, false);
  assert.equal(landmark(again).hasAttribute("hidden"), true);
});

test("the button names the tree, says whether it is shown, and shows and hides it", () => {
  const window = open(true);
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

test("Escape inside the tree hides it and puts the focus on the button", () => {
  const window = open(true);
  button(window).click();
  const item = window.document.querySelector('[role="treeitem"][tabindex="0"]') as unknown as HTMLElement;
  item.focus();
  item.dispatchEvent(new window.KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }) as unknown as Event);
  assert.equal(landmark(window).hasAttribute("hidden"), true);
  assert.equal(button(window).getAttribute("aria-expanded"), "false");
  assert.equal(window.document.activeElement, button(window) as unknown);
});

test("the switch knows whether the focus is in the tree, on the checkbox or on the button, and moves it", () => {
  const window = open(false);
  const toggle = new DomTreeSwitch(window.document as unknown as Document);
  assert.equal(toggle.focus(), null);
  (window.document.querySelector('[role="treeitem"][tabindex="0"]') as unknown as HTMLElement).focus();
  assert.equal(toggle.focus(), "tree");
  toggle.focusCheckbox();
  assert.equal(window.document.activeElement, checkbox(window) as unknown);
  assert.equal(toggle.focus(), "checkbox");
  toggle.focusButton();
  assert.equal(toggle.focus(), "button");
});

test("a control hidden with the focus on it, which drops the focus to the body, is where the focus was", () => {
  const window = open(false);
  const toggle = new DomTreeSwitch(window.document as unknown as Document);
  const box = checkbox(window);
  box.focus();
  // The stylesheet hides it on a narrow window, before the resize is reported.
  box.checkVisibility = () => false;
  box.blur();
  assert.equal(window.document.activeElement, window.document.body as unknown);
  assert.equal(toggle.focus(), "checkbox");
  toggle.focusButton();
  assert.equal(toggle.focus(), "button");
});

test("a control blurred while still shown, as by a click elsewhere, is not where the focus is", () => {
  const window = open(false);
  const toggle = new DomTreeSwitch(window.document as unknown as Document);
  const box = checkbox(window);
  box.focus();
  box.checkVisibility = () => true;
  box.blur();
  assert.equal(toggle.focus(), null);
});

test("data that cannot be loaded leaves the checkbox and the button disabled and the tree hidden and empty", () => {
  const window = open(false, new Map(), null);
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
