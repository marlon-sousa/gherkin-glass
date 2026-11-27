// The policy of src/policies/feature_view.ts, over the store of a feature of each kind of block.

import assert from "node:assert/strict";
import { test } from "node:test";

import { featureView } from "../../src/policies/feature_view.ts";
import { words } from "../support/drawn_pages.ts";
import { refundsDescriptor, refundsEntry, refundsObject } from "../support/refunds_store.ts";

// Rules 42 and 43: a feature's page is titled with its heading and its file's path, in the feature's language.
test("a feature's page is titled with its heading and its file's path, in its language", () => {
  const view = featureView(words, refundsDescriptor, refundsEntry, refundsObject);
  assert.equal(view.title, "Feature: Refunds & <returns>, billing/refunds.feature");
  assert.equal(view.lang, "en");
});
