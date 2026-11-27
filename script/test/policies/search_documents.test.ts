// The policy of src/policies/search_documents.ts, over tests/fixtures/search, against the documents expected of it, tests/fixtures/search/search.json.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { searchDocuments } from "../../src/policies/search_documents.ts";
import type { SearchDocument } from "../../src/ports/documents.ts";
import { searchFeatures } from "../support/search_store.ts";

// Rules 147 and 148: an entry for each feature, rule, background, scenario and examples block, in order, each page named without `.html`.
test("the documents are those expected, each page by its name", () => {
  const expected = JSON.parse(readFileSync(new URL("../../../tests/fixtures/search/search.json", import.meta.url), "utf8")) as SearchDocument[];
  assert.deepEqual(searchDocuments(searchFeatures), expected);
});
