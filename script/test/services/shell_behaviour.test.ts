// The service of src/services/shell_behaviour.ts, with fakes of the drawn page, the folder tree and the browser's memory.

import assert from "node:assert/strict";
import { test } from "node:test";

import type { PageName } from "../../src/policies/address.ts";
import type { DrawnPage } from "../../src/ports/drawn_page.ts";
import type { SearchPage } from "../../src/ports/search_page.ts";
import { shellBehaviour } from "../../src/services/shell_behaviour.ts";
import { FakeBackgrounds, FakeBlock, FakeMemory, FakePage } from "../support/page_fakes.ts";

interface Behaved {
  readonly shown: PageName[];
  readonly filled: string[];
  readonly searched: string[];
  readonly memory: FakeMemory;
  readonly page: FakePage;
}

function behaved(page: PageName, backgrounds: FakeBackgrounds | null = null): Behaved {
  const behaving: Behaved = { shown: [], filled: [], searched: [], memory: new FakeMemory(), page: new FakePage([new FakeBlock("store-credit", null)]) };
  const drawn: DrawnPage = {
    blocks: () => behaving.page,
    backgrounds: () => backgrounds,
    noteDialog: () => null,
    searchPage: (searched): SearchPage | null => {
      behaving.searched.push(searched);
      return null;
    },
  };
  shellBehaviour({
    drawn,
    tree: { show: (shown) => void behaving.shown.push(shown), disable: () => {} },
    fillSearch: (text) => void behaving.filled.push(text),
    documents: { load: async () => [] },
    memory: behaving.memory,
    shellPath: "/site/v1/",
  })(page);
  return behaving;
}

// Rules 89 and 240: the page just drawn is marked in the tree, and its blocks' states are kept under the shell's address of the page.
test("a drawn page is marked in the tree, and the state of its blocks kept under its address in the shell", () => {
  const drawn = behaved({ kind: "feature", page: "billing/refunds" });
  assert.deepEqual(drawn.shown, [{ kind: "feature", page: "billing/refunds" }]);
  drawn.page.toggle(drawn.page.found[0]!);
  assert.deepEqual([...drawn.memory.kept.keys()], ["gherkin-glass:/site/v1/#billing/refunds#store-credit"]);
  assert.deepEqual(drawn.filled, []);
  assert.deepEqual(drawn.searched, []);
});

// Rule 99: the choice of the backgrounds' form is kept for the shell's site, as a written page keeps it for its own.
test("the choice of the backgrounds' form is kept under the shell's folder", () => {
  const backgrounds = new FakeBackgrounds(false);
  const drawn = behaved({ kind: "feature", page: "billing/refunds" }, backgrounds);
  backgrounds.switched(true);
  assert.equal(drawn.memory.kept.get("gherkin-glass:/site/v1/:backgrounds"), "inline");
});

// Rules 216 and 217: the page of search results fills the search box with its words and lists what they find.
test("the page of search results fills the search box with its words and finds its results", () => {
  const drawn = behaved({ kind: "search", words: "open" });
  assert.deepEqual(drawn.filled, ["open"]);
  assert.deepEqual(drawn.searched, ["open"]);
});
