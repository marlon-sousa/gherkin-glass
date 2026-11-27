// The service of src/services/shell_tree.ts, with the store in memory and the tree's adapters on the layout ghg 1 wrote, tests/fixtures/pages/expected/billing/refunds.html: docs/BEHAVIOUR.md, rules 89, 90 and 93.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { Window } from "happy-dom";

import { DomTree } from "../../src/adapters/dom_tree.ts";
import { DomTreeSwitch } from "../../src/adapters/dom_tree_switch.ts";
import type { Memory } from "../../src/ports/memory.ts";
import type { Store } from "../../src/ports/store.ts";
import { startShellTree, type ShellTree } from "../../src/services/shell_tree.ts";
import { billingDescriptor, closedObject } from "../support/billing_store.ts";
import { words } from "../support/drawn_pages.ts";
import { MemoryStore } from "../support/memory_store.ts";
import { refundsObject } from "../support/refunds_store.ts";
import { settled } from "../support/shell_fakes.ts";

const LAYOUT = readFileSync(new URL("../../../tests/fixtures/pages/expected/billing/refunds.html", import.meta.url), "utf8");

function shell(store: Store, kept = new Map<string, string>()): { readonly document: Document; readonly tree: ShellTree } {
  const window = new Window({ url: "https://example.com/site/index.html#billing/refunds" });
  window.document.write(LAYOUT);
  const document = window.document as unknown as Document;
  document.querySelector("nav.ghg-tree")!.replaceChildren();
  const memory: Memory = { get: (key) => kept.get(key) ?? null, set: (key, value) => void kept.set(key, value) };
  const tree = startShellTree({
    store,
    words,
    toggle: new DomTreeSwitch(document),
    view: DomTree.inShell(document),
    width: { narrow: () => false, onChange: () => {} },
    memory,
    sitePath: "/site/",
  });
  return { document, tree };
}

const billingStore = (): MemoryStore => new MemoryStore(billingDescriptor, { refunds: refundsObject, "closed-1": closedObject });
const current = (document: Document): string | null | undefined => document.querySelector('nav.ghg-tree [aria-current="page"]')?.getAttribute("href");
const expanded = (document: Document): (string | null)[] => [...document.querySelectorAll('[role="treeitem"][aria-expanded="true"]')].map((item) => item.getAttribute("href"));

// Rules 89 and 90: every item links to the shell's own address, and the page drawn is marked, with the items it is inside expanded.
test("the tree marks the page drawn, each time one is", async () => {
  const { document, tree } = shell(billingStore());
  tree.show({ kind: "feature", page: "billing/refunds" });
  await settled();
  assert.equal(current(document), "#billing/refunds");
  assert.deepEqual(expanded(document), ["#billing/", "#billing/refunds"]);
  assert.equal(document.querySelector('[role="treeitem"]')?.getAttribute("href"), "#");
  tree.show({ kind: "feature", page: "billing/archive/closed #1" });
  assert.equal(current(document), "#billing/archive/closed%20%231");
  assert.deepEqual(expanded(document), ["#billing/", "#billing/archive/", "#billing/archive/closed%20%231"]);
  // Rule 91: the tree's one stop of the Tab key is the current page's item.
  assert.deepEqual(
    [...document.querySelectorAll<HTMLElement>('[role="treeitem"]')].filter((item) => item.tabIndex === 0).map((item) => item.getAttribute("href")),
    ["#billing/archive/closed%20%231"],
  );
  tree.show({ kind: "tags" });
  assert.equal(current(document), undefined);
  assert.deepEqual(expanded(document), []);
});

// Rule 93: the tree stays empty and its controls are disabled when an object cannot be had.
test("the tree's controls are disabled when an object cannot be fetched", async () => {
  const { document, tree } = shell(new MemoryStore(billingDescriptor, { refunds: refundsObject }));
  tree.show({ kind: "home" });
  await settled();
  assert.equal(document.querySelector('[role="treeitem"]'), null);
  assert.equal(document.querySelector<HTMLInputElement>("input.ghg-show-tree")!.disabled, true);
});

// Rule 243: a page that cannot be loaded disables the tree's controls too.
test("the tree's controls are disabled when asked", async () => {
  const { document, tree } = shell(billingStore());
  tree.show({ kind: "home" });
  await settled();
  tree.disable();
  assert.equal(document.querySelector<HTMLInputElement>("input.ghg-show-tree")!.disabled, true);
  assert.equal(document.querySelector<HTMLElement>("nav.ghg-tree")!.hidden, true);
});
