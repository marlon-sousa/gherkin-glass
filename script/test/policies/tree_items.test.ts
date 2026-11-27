// The folder tree's items of src/policies/tree_items.ts, against the tree ghg 1 wrote.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { pathToPage, treeItems } from "../../src/policies/tree_items.ts";
import type { TreeItem } from "../../src/ports/tree.ts";
import { billingDescriptor, billingFeatures } from "../support/billing_store.ts";
import { newAddress, words } from "../support/drawn_pages.ts";

/** ghg 1's tree of tests/fixtures/pages, each address written as rule 235 writes it. */
function writtenTree(): TreeItem[] {
  const renamed = (item: TreeItem): TreeItem => ({ ...item, address: newAddress(item.address, "index.html"), ...(item.items && { items: item.items.map(renamed) }) });
  const items = JSON.parse(readFileSync(new URL("../../../tests/fixtures/pages/expected/assets/tree.json", import.meta.url), "utf8")) as TreeItem[];
  return items.map(renamed);
}

/** Without the empty lists of items that hold nothing, which ghg 1 left out. */
function withoutEmpty(item: TreeItem): TreeItem {
  const { items, ...rest } = item;
  return items === undefined || items.length === 0 ? rest : { ...rest, items: items.map(withoutEmpty) };
}

// Rule 89: the home page, then the folders and the features, each feature's contents inside it.
test("the tree holds the items ghg 1 wrote, but for their addresses", () => {
  const items = treeItems(words, billingDescriptor, billingFeatures).map(withoutEmpty);
  const written = writtenTree();
  assert.deepEqual(items, [written[0], written.find((item) => item.address === "#billing/")]);
});

// Rule 90: the current page's item and every item it is inside.
test("the path to a page runs from its item out", () => {
  const items = treeItems(words, billingDescriptor, billingFeatures);
  assert.deepEqual(pathToPage(items, { kind: "feature", page: "billing/archive/closed #1" }), ["#billing/archive/closed%20%231", "#billing/archive/", "#billing/"]);
  assert.deepEqual(pathToPage(items, { kind: "home" }), ["#"]);
  assert.deepEqual(pathToPage(items, { kind: "tags" }), []);
});
