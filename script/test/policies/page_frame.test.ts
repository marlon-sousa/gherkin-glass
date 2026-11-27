// The policy of src/policies/page_frame.ts: the frame of a page of a version, which names the version and starts at the versions page.

import assert from "node:assert/strict";
import { test } from "node:test";

import { frameOf, homeCrumb, missingFrame, versionedFrame, versionWords } from "../../src/policies/page_frame.ts";
import { plain } from "../../src/policies/heading_words.ts";
import type { VersionKind } from "../../src/ports/store.ts";
import { words } from "../support/drawn_pages.ts";
import { descriptor } from "../support/store_building.ts";

function versionOf(name: string, kind: VersionKind): string | null {
  return versionWords(words, descriptor("Banking", [], { version: name, version_kind: kind }));
}

// Rule 268: a development version and a pre-release say what they are, and a release reads its name alone.
test("a version is worded from the kind its descriptor gives", () => {
  assert.equal(versionOf("main", "development"), "main, a development version");
  assert.equal(versionOf("v2.0.0-rc.1", "pre-release"), "v2.0.0-rc.1, a pre-release");
  assert.equal(versionOf("v1.0.0", "release"), "v1.0.0");
});

// Rule 268: a site without versions and a preview say nothing of a version.
test("a site without versions and a preview name no version", () => {
  assert.equal(versionWords(words, descriptor("Banking", [])), null);
  assert.equal(versionWords(words, descriptor("Banking", [], { preview: "pr-1" })), null);
  const frame = frameOf("Banking", "Tags", "en", [], { kind: "tags" }, plain("Tags"));
  assert.deepEqual(versionedFrame(words, null, frame), frame);
});

// Rules 267 and 269: a feature's page of a version starts its breadcrumb at the versions page, then the version, and ends its title with the version.
test("a page of a version starts its breadcrumb at the versions page and ends its title with the version", () => {
  const frame = frameOf("Banking", "Feature: Refunds, billing/refunds.feature", "en", [], { kind: "feature", page: "billing/refunds" }, plain("Feature: Refunds"));
  const versioned = versionedFrame(words, "main, a development version", frame);
  assert.equal(versioned.title, "Feature: Refunds, billing/refunds.feature, main, a development version");
  assert.deepEqual(
    versioned.crumbs.map((crumb) => [crumb.address, crumb.words.map((part) => part.text).join("")]),
    [
      ["../index.html", "Versions"],
      ["#", "main, a development version"],
      ["#billing/refunds", "Feature: Refunds"],
    ],
  );
});

// Rule 267: on the version's home page, its own crumb reads the version.
test("a version's home page has the versions page and itself, which reads the version", () => {
  const home = { title: "Banking", lang: "en", crumbs: [homeCrumb("Banking")] };
  const versioned = versionedFrame(words, "v1.0.0", home);
  assert.equal(versioned.title, "Banking, v1.0.0");
  assert.deepEqual(
    versioned.crumbs.map((crumb) => crumb.words.map((part) => part.text).join("")),
    ["Versions", "v1.0.0"],
  );
});

// Rules 239 and 269: the page of an address that names no page names the version too.
test("the page of an address that names no page of a version names the version", () => {
  const versioned = versionedFrame(words, "v1.0.0", missingFrame(words, "Banking"));
  assert.equal(versioned.title, "Nothing is at this address, Banking, v1.0.0");
  assert.equal(versioned.crumbs.length, 2);
});
