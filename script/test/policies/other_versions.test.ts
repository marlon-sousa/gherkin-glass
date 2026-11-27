// The policy of src/policies/other_versions.ts: docs/BEHAVIOUR.md, rules 259 to 262.

import assert from "node:assert/strict";
import { test } from "node:test";

import { type OtherVersionsView, otherObjects, pageInVersions } from "../../src/policies/other_versions.ts";
import type { PageIndex, VersionList } from "../../src/ports/store.ts";
import { words } from "../support/drawn_pages.ts";
import { block, descriptor, object } from "../support/store_building.ts";

const feature = (...blocks: ReturnType<typeof block>[]) => object(block("feature", "Feature", "Refunds", "", { hash: "feature", blocks }));
const scenario = (anchor: string, hash: string) => block("scenario", "Scenario", anchor, anchor, { hash });

const shown = feature(scenario("s1", "h1"), block("rule", "Rule", "r", "r", { hash: "hr", blocks: [scenario("s2", "h2")] }));
// Changed: s1 changed, s2 and its rule not there, s3 only there.
const changed = feature(scenario("s1", "h1-changed"), scenario("s3", "h3"));
// The same hashes in another object, as when blocks only moved.
const moved = feature(scenario("s1", "h1"), block("rule", "Rule", "r", "r", { hash: "hr", blocks: [scenario("s2", "h2")] }));

const LIST: VersionList = { format: 1, development: ["main"], releases: ["v2.0.0", "v1.0.0", "v0.9.0"], pre_releases: ["v2.0.0-rc.1"], precedence: [["v2.0.0"], ["v2.0.0-rc.1"], ["v1.0.0"], ["v0.9.0"]] };

const INDEX: PageIndex = {
  format: 1,
  pages: [
    {
      page: "billing/refunds",
      objects: [
        { object: "a", versions: ["main", "v1.0.0"] },
        { object: "b", versions: ["v2.0.0"] },
        { object: "c", versions: ["v0.9.0"] },
      ],
    },
  ],
};

const OTHERS = new Map([
  ["b", changed],
  ["c", moved],
]);

/** Each group's label, and its links' addresses, texts and what follows them. */
function shownLists(view: OtherVersionsView | null | undefined): unknown {
  return view?.groups.map((group) => [group.label, group.links.map((link) => [link.address, link.text, link.item.map((piece) => ("slot" in piece ? "<link>" : piece.words)).join("")])]);
}

test("the objects needed are those other versions have for the page", () => {
  assert.deepEqual(otherObjects(INDEX, "billing/refunds", "a"), ["b", "c"]);
  assert.deepEqual(otherObjects(INDEX, "nowhere", "a"), []);
});

// Rules 259, 261 and 262: the page's groups, in order, the versions in the order of rule 127, newer ones said so.
test("a version's page lists the versions it changed in, is not in and is unchanged in", () => {
  const lists = pageInVersions(words, descriptor("Site", [], { version: "v1.0.0" }), LIST, INDEX, "billing/refunds", { name: "a", object: shown }, OTHERS);
  assert.equal(lists.page?.label, "This page in other versions");
  assert.deepEqual(shownLists(lists.page), [
    ["Changed in", [["../v2.0.0/index.html#billing/refunds", "v2.0.0, newer", "<link>: 1 block changed, 2 not there, 1 only there"]]],
    ["Not in", [["../v2.0.0-rc.1/index.html", "v2.0.0-rc.1, newer", "<link>"]]],
    [
      "Unchanged in",
      [
        ["../main/index.html#billing/refunds", "main", "<link>"],
        ["../v0.9.0/index.html#billing/refunds", "v0.9.0", "<link>"],
      ],
    ],
  ]);
});

// Rules 260 and 262: each block's groups, by its anchor and hash, its links to the block, the page or the home page.
test("each block lists the versions it changed in, is not in and is unchanged in", () => {
  const lists = pageInVersions(words, descriptor("Site", [], { version: "v1.0.0" }), LIST, INDEX, "billing/refunds", { name: "a", object: shown }, OTHERS);
  assert.deepEqual([...lists.blocks.keys()], ["s1", "r", "s2"]);
  assert.equal(lists.blocks.get("s1")?.label, "In other versions");
  assert.deepEqual(shownLists(lists.blocks.get("s1")), [
    ["Changed in", [["../v2.0.0/index.html#billing/refunds#s1", "v2.0.0, newer", "<link>"]]],
    ["Not in", [["../v2.0.0-rc.1/index.html", "v2.0.0-rc.1, newer", "<link>"]]],
    [
      "Unchanged in",
      [
        ["../main/index.html#billing/refunds#s1", "main", "<link>"],
        ["../v0.9.0/index.html#billing/refunds#s1", "v0.9.0", "<link>"],
      ],
    ],
  ]);
  assert.deepEqual(shownLists(lists.blocks.get("s2")), [
    [
      "Not in",
      [
        ["../v2.0.0/index.html#billing/refunds", "v2.0.0, newer", "<link>"],
        ["../v2.0.0-rc.1/index.html", "v2.0.0-rc.1, newer", "<link>"],
      ],
    ],
    [
      "Unchanged in",
      [
        ["../main/index.html#billing/refunds#s2", "main", "<link>"],
        ["../v0.9.0/index.html#billing/refunds#s2", "v0.9.0", "<link>"],
      ],
    ],
  ]);
});

// Rule 262: a preview lists every version, from two folders deeper, none of them newer than a preview.
test("a preview lists every version, none newer", () => {
  const lists = pageInVersions(words, descriptor("Site", [], { preview: "pr-1" }), LIST, INDEX, "billing/refunds", { name: "a", object: shown }, OTHERS);
  const links = lists.page?.groups.flatMap((group) => group.links.map((link) => [link.address, link.text]));
  assert.deepEqual(links, [
    ["../../v2.0.0/index.html#billing/refunds", "v2.0.0"],
    ["../../v2.0.0-rc.1/index.html", "v2.0.0-rc.1"],
    ["../../main/index.html#billing/refunds", "main"],
    ["../../v1.0.0/index.html#billing/refunds", "v1.0.0"],
    ["../../v0.9.0/index.html#billing/refunds", "v0.9.0"],
  ]);
});

// Rule 259: a page with no other version to list has no lists.
test("a page of the only version has no lists", () => {
  const alone: VersionList = { format: 1, development: [], releases: ["v1.0.0"], pre_releases: [], precedence: [["v1.0.0"]] };
  const lists = pageInVersions(words, descriptor("Site", [], { version: "v1.0.0" }), alone, INDEX, "billing/refunds", { name: "a", object: shown }, OTHERS);
  assert.equal(lists.page, null);
  assert.equal(lists.blocks.size, 0);
});
