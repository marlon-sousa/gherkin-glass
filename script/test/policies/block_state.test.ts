// The policy of src/policies/block_state.ts.

import assert from "node:assert/strict";
import { test } from "node:test";

import { formStart, inlineMemoryKey, memoryKey, remembered, startState, switchedState } from "../../src/policies/block_state.ts";

test("a remembered state wins over the configured one", () => {
  assert.equal(startState(true, "collapsed"), false);
  assert.equal(startState(false, "expanded"), true);
  assert.equal(startState(false, null), false);
  assert.equal(startState(true, "something else"), true);
});

test("a state is kept by page and anchor", () => {
  assert.equal(memoryKey("/site/billing/refunds.html", "store-credit"), "gherkin-glass:/site/billing/refunds.html#store-credit");
  assert.notEqual(memoryKey("/a.html", "x"), memoryKey("/b.html", "x"));
  assert.equal(startState(false, remembered(true)), true);
  assert.equal(startState(true, remembered(false)), false);
});

test("a background the checkbox sets is kept apart for the checked box", () => {
  assert.equal(inlineMemoryKey("/site/a.html", "background"), "gherkin-glass:inline:/site/a.html#background");
  assert.notEqual(inlineMemoryKey("/site/a.html", "background"), memoryKey("/site/a.html", "background"));
});

test("the checked box starts the backgrounds inside the scenarios expanded and those in their own place collapsed, and any other block as it was written", () => {
  assert.equal(formStart(true, true, false, true), false);
  assert.equal(formStart(false, false, true, true), true);
  assert.equal(formStart(false, true, false, false), true);
  assert.equal(formStart(true, false, false, false), false);
  assert.equal(formStart(true, false, true, false), false);
  assert.equal(formStart(true, null, false, true), true);
  assert.equal(formStart(false, null, false, false), false);
});

test("checking the box expands the backgrounds inside the scenarios and collapses those in their own place, and unchecking it does the opposite", () => {
  assert.equal(switchedState(true, true), true);
  assert.equal(switchedState(false, true), false);
  assert.equal(switchedState(true, false), false);
  assert.equal(switchedState(false, false), true);
});
