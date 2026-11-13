// The policy of src/policies/tree_state.ts: docs/BEHAVIOUR.md, rules 87 to 90 and 93.

import assert from "node:assert/strict";
import { test } from "node:test";

import { pageAddress, pathTo, readTree, rootFrom, startChecked, startExpanded, treeShown } from "../../src/policies/tree_state.ts";

test("the reader's choice wins over the configuration for the checkbox", () => {
  assert.equal(startChecked(true, null), true);
  assert.equal(startChecked(false, null), false);
  assert.equal(startChecked(false, "shown"), true);
  assert.equal(startChecked(true, "hidden"), false);
  assert.equal(startChecked(true, "something else"), true);
});

test("a narrow window shows the tree as the button says, any other as the checkbox does", () => {
  assert.equal(treeShown(false, true, false), true);
  assert.equal(treeShown(false, false, true), false);
  assert.equal(treeShown(true, true, false), false);
  assert.equal(treeShown(true, false, true), true);
});

test("an item on the path to the current page is expanded whatever was remembered", () => {
  assert.equal(startExpanded(true, "collapsed"), true);
  assert.equal(startExpanded(false, null), false);
  assert.equal(startExpanded(false, "expanded"), true);
  assert.equal(startExpanded(false, "collapsed"), false);
});

test("the current page's address below the site's root is decoded, with the folder page named", () => {
  assert.equal(pageAddress("/site/", "/site/billing/closed%20%231.html"), "billing/closed #1.html");
  assert.equal(pageAddress("/site/", "/site/"), "index.html");
  assert.equal(pageAddress("/site/", "/site/billing/"), "billing/index.html");
  assert.equal(pageAddress("/site/", "/elsewhere/a.html"), null);
  assert.equal(rootFrom("billing/archive/index.html"), "../../");
  assert.equal(rootFrom("index.html"), "");
});

test("the path to a page is its own item and every item it is inside, but never a block's", () => {
  const items = [
    { text: "Home", address: "index.html" },
    {
      text: "a b",
      address: "a%20b/index.html",
      items: [{ text: "F", address: "a%20b/f.html", items: [{ text: "S", address: "a%20b/f.html#s" }] }],
    },
  ];
  assert.deepEqual(pathTo(items, "a b/f.html"), ["a%20b/f.html", "a%20b/index.html"]);
  assert.deepEqual(pathTo(items, "index.html"), ["index.html"]);
  assert.deepEqual(pathTo(items, "missing.html"), []);
});

test("data that is not a list of items is no tree", () => {
  assert.deepEqual(readTree([{ text: "a", address: "a.html", lang: "pt", items: [] }]), [{ text: "a", address: "a.html", lang: "pt", items: [] }]);
  assert.equal(readTree({ items: [] }), null);
  assert.equal(readTree([{ text: "a" }]), null);
  assert.equal(readTree([{ text: "a", address: "a.html", items: [{ text: 1, address: "b" }] }]), null);
});
