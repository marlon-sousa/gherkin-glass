// The policy of src/policies/tree_state.ts: docs/BEHAVIOUR.md, rules 87, 88, 90 and 93.

import assert from "node:assert/strict";
import { test } from "node:test";

import { focusHidden, startChecked, startExpanded, treeShown } from "../../src/policies/tree_state.ts";

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

test("a resize hides the tree once it is not shown, the checkbox on a narrow window and the button on a wide one", () => {
  assert.equal(focusHidden("tree", true, false), true);
  assert.equal(focusHidden("tree", false, true), false);
  assert.equal(focusHidden("checkbox", true, true), true);
  assert.equal(focusHidden("checkbox", false, false), false);
  assert.equal(focusHidden("button", false, true), true);
  assert.equal(focusHidden("button", true, false), false);
  assert.equal(focusHidden(null, true, false), false);
});
