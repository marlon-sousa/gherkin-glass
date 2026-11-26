// The adapter of src/adapters/fluent_words.ts, over the catalogue, messages/en/gherkin-glass.ftl.

import assert from "node:assert/strict";
import { test } from "node:test";

import { FluentWords } from "../../src/adapters/fluent_words.ts";
import { words } from "../support/drawn_pages.ts";

// Rule 102: a count is a plural form of its message, an exact number first.
test("a count is worded by the catalogue's plural forms", () => {
  assert.equal(words.text("tag-index-entry", { tag: "@draft", count: 0 }), "@draft, no scenarios");
  assert.equal(words.text("tag-index-entry", { tag: "@billing", count: 1 }), "@billing, one scenario");
  assert.equal(words.text("tag-index-entry", { tag: "@slow", count: 3 }), "@slow, 3 scenarios");
});

// Rule 43: feature text in a message is isolated, and no isolation mark is written around it.
test("feature text in a message is a part of its own, isolated", () => {
  assert.deepEqual(words.parts("named-heading", { kind: "scenario-outline" }, { name: [{ text: "Withdraw", isolated: true }] }), [
    { text: "Scenario outline: ", isolated: false },
    { text: "Withdraw", isolated: true },
  ]);
  assert.equal(words.text("page-title", { heading: "Feature: A", path: "a.feature" }), "Feature: A, a.feature");
});

test("a message the catalogue does not hold reads its id", () => {
  assert.equal(new FluentWords("").text("collapse-all"), "collapse-all");
});
