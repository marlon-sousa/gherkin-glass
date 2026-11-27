// The policy of src/policies/address.ts.

import assert from "node:assert/strict";
import { test } from "node:test";

import { encoded, readTarget, targetAddress, type PageName } from "../../src/policies/address.ts";

const page = (name: PageName, anchor: string | null = null) => ({ kind: "page", page: name, anchor });

// Rule 235: a page's name, then a block's anchor after a second `#`.
test("an address names a feature's page, and a block of it after a second #", () => {
  assert.deepEqual(readTarget("#catalogue/searching#searching-by-title"), page({ kind: "feature", page: "catalogue/searching" }, "searching-by-title"));
  assert.deepEqual(readTarget("#catalogue/searching"), page({ kind: "feature", page: "catalogue/searching" }));
  assert.deepEqual(readTarget("#catalogue/searching#"), page({ kind: "feature", page: "catalogue/searching" }));
});

// Rule 235: a folder's page ends with `/`, and the home page's name is empty.
test("a folder's page ends with a slash, and the home page has nothing after the #, or no #", () => {
  assert.deepEqual(readTarget("#catalogue/archive/"), page({ kind: "folder", folder: "catalogue/archive/" }));
  assert.deepEqual(readTarget(""), page({ kind: "home" }));
  assert.deepEqual(readTarget("#"), page({ kind: "home" }));
});

// Rule 236: `##` names an element of the page shown.
test("two # in a row name an element of the page shown", () => {
  assert.deepEqual(readTarget("##ghg_main"), { kind: "element", id: "ghg_main" });
});

// Rule 237: the site's own pages.
test("the tags, a tag, the step sentences and the search results have names of their own", () => {
  assert.deepEqual(readTarget("#tags/"), page({ kind: "tags" }));
  assert.deepEqual(readTarget("#tags/needs-db"), page({ kind: "tag", tag: "needs-db" }));
  assert.deepEqual(readTarget("#steps"), page({ kind: "steps" }));
  assert.deepEqual(readTarget("#search?q=free+room"), page({ kind: "search", words: "free room" }));
  assert.deepEqual(readTarget("#search"), page({ kind: "search", words: "" }));
  assert.equal(targetAddress({ kind: "tags" }), "#tags/");
  assert.equal(targetAddress({ kind: "tag", tag: "slow-2" }), "#tags/slow-2");
  assert.equal(targetAddress({ kind: "steps" }), "#steps");
  assert.equal(targetAddress({ kind: "search", words: "free room" }), "#search?q=free+room");
  assert.equal(targetAddress({ kind: "search", words: "" }), "#search");
});

// Rule 238: every byte but a letter, a digit, `-`, `.`, `_`, `~` and `/` is percent-encoded.
test("a name and an anchor are percent-encoded byte by byte", () => {
  assert.equal(targetAddress({ kind: "feature", page: "saque em conta poupança" }), "#saque%20em%20conta%20poupan%C3%A7a");
  assert.equal(targetAddress({ kind: "feature", page: "a/b" }, "saque-em-conta-poupança"), "#a/b#saque-em-conta-poupan%C3%A7a");
  assert.equal(encoded("it's (a)!*#?&+"), "it%27s%20%28a%29%21%2A%23%3F%26%2B");
  assert.equal(targetAddress({ kind: "folder", folder: "x y/" }), "#x%20y/");
  assert.equal(targetAddress({ kind: "home" }), "#");
});

// Rule 238: an address is decoded as UTF-8, and read as typed in the address bar too.
test("an address is read decoded, whether or not its characters are encoded", () => {
  const portuguese = page({ kind: "feature", page: "saque em conta poupança" }, "saque");
  assert.deepEqual(readTarget("#saque%20em%20conta%20poupan%C3%A7a#saque"), portuguese);
  assert.deepEqual(readTarget("#saque em conta poupança#saque"), portuguese);
  assert.deepEqual(readTarget("#search?q=poupan%C3%A7a%20livre"), page({ kind: "search", words: "poupança livre" }));
  assert.deepEqual(readTarget("#100%"), page({ kind: "feature", page: "100%" }));
});

// The home page has no anchor of its own, since `##` names an element of the page shown.
test("what an address names is written back as the same address", () => {
  assert.deepEqual(readTarget(targetAddress({ kind: "home" })), page({ kind: "home" }));
  const names: PageName[] = [
    { kind: "folder", folder: "billing/archive/" },
    { kind: "feature", page: "saque em conta poupança" },
    { kind: "tags" },
    { kind: "tag", tag: "slow" },
    { kind: "steps" },
    { kind: "search", words: "free room & more" },
  ];
  for (const name of names) {
    assert.deepEqual(readTarget(targetAddress(name, "a b")), page(name, "a b"));
  }
});
