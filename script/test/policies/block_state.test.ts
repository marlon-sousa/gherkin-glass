// The policy of src/policies/block_state.ts.

import assert from "node:assert/strict";
import { test } from "node:test";

import { memoryKey, remembered, startState } from "../../src/policies/block_state.ts";

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
