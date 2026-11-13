// The policy of src/policies/block_state.ts.

import assert from "node:assert/strict";
import { test } from "node:test";

import { formStart, inlineMemoryKey, memoryKey, remembered, startState } from "../../src/policies/block_state.ts";

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

test("a background whose steps a scenario starts with is kept apart for the inline form", () => {
  assert.equal(inlineMemoryKey("/site/a.html", "background"), "gherkin-glass:inline:/site/a.html#background");
  assert.notEqual(inlineMemoryKey("/site/a.html", "background"), memoryKey("/site/a.html", "background"));
});

test("a background whose steps a scenario starts with starts collapsed with the inline form, and any other block as it was written", () => {
  assert.equal(formStart(true, true, true), false);
  assert.equal(formStart(false, true, false), true);
  assert.equal(formStart(true, false, false), false);
  assert.equal(formStart(true, null, true), true);
  assert.equal(formStart(false, null, false), false);
});
