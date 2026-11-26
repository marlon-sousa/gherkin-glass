// The policy of src/policies/store_format.ts.

import assert from "node:assert/strict";
import { test } from "node:test";

import { readDescriptor, readObject, readPages, readVersions } from "../../src/policies/store_format.ts";
import { outlinesDescriptor, outlinesObject } from "../support/outlines_store.ts";

/** `value` as it comes back from JSON. */
const parsed = (value: unknown): unknown => JSON.parse(JSON.stringify(value));

// Rules 248 to 252: every member the script reads is read as the rules describe it.
test("an object and a descriptor are read member by member", () => {
  assert.deepEqual(readObject(parsed(outlinesObject)), outlinesObject);
  assert.deepEqual(readDescriptor(parsed(outlinesDescriptor)), outlinesDescriptor);
});

// Rule 245: a reader passes over the members it does not know.
test("a member the script does not know is passed over", () => {
  const descriptor = { ...outlinesDescriptor, settings: { title: null }, front_page: null, later: [1, 2] };
  assert.deepEqual(readDescriptor(parsed(descriptor)), outlinesDescriptor);
});

// Rules 93 and 243: data that is not what the script expects is refused, whole.
test("a missing member, a member of another kind and another format are refused", () => {
  const { lang: _lang, ...withoutLang } = outlinesObject;
  assert.throws(() => readObject(parsed(withoutLang)), TypeError);
  assert.throws(() => readObject(parsed({ ...outlinesObject, format: 2 })), TypeError);
  assert.throws(() => readDescriptor(parsed({ ...outlinesDescriptor, only: "no" })), TypeError);
  assert.throws(() => readObject(parsed({ ...outlinesObject, feature: { ...outlinesObject.feature, kind: "story" } })), TypeError);
  assert.throws(() => readObject(null), TypeError);
  assert.throws(() => readObject([]), TypeError);
});

// Rules 253 and 254.
test("the list of versions and the index of pages are read", () => {
  const versions = { format: 1, development: ["main"], releases: ["v1.1.0", "v1.0.0"], pre_releases: [], precedence: [["v1.1.0"], ["v1.0.0"]] };
  assert.deepEqual(readVersions(parsed({ ...versions, versions_page: { ghg: "2.0.0" } })), versions);
  const pages = { format: 1, pages: [{ page: "a", objects: [{ object: "3f", versions: ["main", "v1.0.0"] }] }] };
  assert.deepEqual(readPages(parsed(pages)), pages);
  assert.throws(() => readPages(parsed({ format: 1, pages: [{ page: "a", objects: [{ object: "3f", versions: [1] }] }] })), TypeError);
});
