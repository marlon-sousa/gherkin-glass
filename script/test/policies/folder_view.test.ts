// The policy of src/policies/folder_view.ts, over the store of a site of two folders.

import assert from "node:assert/strict";
import { test } from "node:test";

import { folderView } from "../../src/policies/folder_view.ts";
import { billingDescriptor, billingFeatures } from "../support/billing_store.ts";
import { words } from "../support/drawn_pages.ts";

// Rule 81: a folder's page is headed with its name and titled with its path and the site's title.
test("a folder's page is headed with its name, and titled with its path and the site's title", () => {
  const view = folderView(words, billingDescriptor, billingFeatures, "billing/archive/")!;
  assert.equal(view.heading, "archive");
  assert.equal(view.title, "billing/archive, Banking & <more>");
  assert.deepEqual(view.totals, []);
});

// Rule 77: the home page is the root folder's, headed and titled with the site's title, with its totals.
test("the home page is headed and titled with the site's title, and holds the site's totals", () => {
  const view = folderView(words, billingDescriptor, billingFeatures, "")!;
  assert.equal(view.heading, "Banking & <more>");
  assert.equal(view.title, "Banking & <more>");
  assert.notDeepEqual(view.totals, []);
});

// Rule 77: a folder no feature is in has no page.
test("a folder no feature is in has no page", () => {
  assert.equal(folderView(words, billingDescriptor, billingFeatures, "nowhere/"), null);
});
