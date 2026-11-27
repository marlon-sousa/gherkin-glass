// The service of src/services/results_listing.ts, with fake ports, over tests/fixtures/search: docs/BEHAVIOUR.md, rules 216 and 218.

import assert from "node:assert/strict";
import { test } from "node:test";

import { startListing } from "../../src/services/results_listing.ts";
import type { Documents, SearchDocument } from "../../src/ports/documents.ts";
import type { PageWords, Part, ResultGroup, SearchPage } from "../../src/ports/search_page.ts";
import { documents, searchPageWords } from "../support/search_fixtures.ts";

class FakePage implements SearchPage {
  statuses: string[] = [];
  field = "";
  heading: readonly Part[] = [];
  title = "";
  groups: readonly ResultGroup[] | null = null;

  constructor(readonly address: string) {}

  words(): PageWords {
    return searchPageWords();
  }
  searched(): string {
    return this.address;
  }
  showSearched(words: string, heading: readonly Part[], title: string): void {
    this.field = words;
    this.heading = heading;
    this.title = title;
  }
  setStatus(text: string): void {
    this.statuses.push(text);
  }
  showGroups(groups: readonly ResultGroup[]): void {
    this.groups = groups;
  }
}

const fetched = (found: readonly SearchDocument[]): Documents => ({ load: () => Promise.resolve(found) });

test("the words of the address are searched, and every result is listed", async () => {
  const many = Array.from({ length: 60 }, (_, number) => ({ ...documents()[2]!, anchor: `withdraw-${number}` }));
  const page = new FakePage("  withdraw ");
  await startListing(page, fetched(many));
  assert.equal(page.field, "withdraw");
  assert.equal(page.title, 'Results for "withdraw", Banking & <more>');
  assert.equal(page.groups?.[0]?.results.length, 60);
  assert.deepEqual(page.statuses, ["Loading search.", "Results: 60. Features: 1."]);
});

test("an address without words, or with only spaces, searches nothing and says nothing", async () => {
  let loads = 0;
  const counted: Documents = {
    load: () => {
      loads += 1;
      return Promise.resolve(documents());
    },
  };
  for (const words of ["", "   "]) {
    const page = new FakePage(words);
    await startListing(page, counted);
    assert.deepEqual(page.statuses, []);
    assert.equal(page.groups, null);
    assert.equal(page.title, "");
  }
  assert.equal(loads, 0);
});

test("documents that cannot be fetched or read say so", async () => {
  for (const documents of [
    { load: () => Promise.reject(new Error("offline")) },
    { load: () => Promise.resolve({} as unknown as SearchDocument[]) },
  ]) {
    const page = new FakePage("withdraw");
    await startListing(page, documents);
    assert.deepEqual(page.statuses, ["Loading search.", "Search could not be loaded."]);
    assert.equal(page.groups, null);
  }
});

test("a search with no result says so", async () => {
  const page = new FakePage("zebra");
  await startListing(page, fetched(documents()));
  assert.deepEqual(page.groups, []);
  assert.equal(page.statuses.at(-1), "No results.");
});
