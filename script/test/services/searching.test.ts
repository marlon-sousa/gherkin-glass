// The service of src/services/searching.ts, with fake ports, over tests/fixtures/search: docs/BEHAVIOUR.md, rules 152 to 156 and 214.

import assert from "node:assert/strict";
import { test } from "node:test";

import { startSearching } from "../../src/services/searching.ts";
import type { Documents, SearchDocument } from "../../src/ports/documents.ts";
import type { Result, SearchBox, SearchWords } from "../../src/ports/search_box.ts";
import type { Timer } from "../../src/ports/timer.ts";
import { documents, pageWords } from "../support/search_fixtures.ts";

class FakeBox implements SearchBox {
  readonly given: SearchWords;
  status = "";
  results: readonly Result[] = [];
  field = "";
  open = false;
  chosen: number | null = null;
  link: Result | null = null;
  went: string | null = null;
  input: (text: string) => void = () => {};
  down: () => void = () => {};
  up: () => void = () => {};
  enter: () => void = () => {};
  escape: () => void = () => {};
  pick: (place: number) => void = () => {};

  constructor(words: SearchWords) {
    this.given = words;
  }

  words(): SearchWords {
    return this.given;
  }
  fieldText(): string {
    return this.field;
  }
  onInput(handler: (text: string) => void): void {
    this.input = (text) => {
      this.field = text;
      handler(text);
    };
  }
  onDown(handler: () => void): void {
    this.down = handler;
  }
  onUp(handler: () => void): void {
    this.up = handler;
  }
  onEnter(handler: () => void): void {
    this.enter = handler;
  }
  onEscape(handler: () => void): void {
    this.escape = handler;
  }
  onPick(handler: (place: number) => void): void {
    this.pick = handler;
  }
  setStatus(text: string): void {
    this.status = text;
  }
  showResults(results: readonly Result[]): void {
    this.results = results;
    this.chosen = null;
  }
  setOpen(open: boolean): void {
    this.open = open;
  }
  choose(place: number | null): void {
    this.chosen = place;
  }
  showAllResults(link: Result | null): void {
    this.link = link;
  }
  fill(text: string): void {
    this.field = text;
  }
  go(address: string): void {
    this.went = address;
  }

  addresses(): string[] {
    return this.results.map((found) => found.address);
  }
}

class FakeDocuments implements Documents {
  loads = 0;
  failing: boolean;
  readonly found: SearchDocument[];

  constructor(found: SearchDocument[], failing = false) {
    this.found = found;
    this.failing = failing;
  }

  load(): Promise<readonly SearchDocument[]> {
    this.loads += 1;
    return this.failing ? Promise.reject(new Error("offline")) : Promise.resolve(this.found);
  }
}

/** Calls nothing until `pass` is called, as if the reader had paused. */
class FakeTimer implements Timer {
  waited: number | null = null;
  private handler: (() => void) | null = null;

  after(milliseconds: number, handler: () => void): void {
    this.waited = milliseconds;
    this.handler = handler;
  }
  cancel(): void {
    this.handler = null;
  }
  pass(): void {
    const handler = this.handler;
    this.handler = null;
    handler?.();
  }
}

const settled = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

/** The service started over a box whose field already holds `filled`. */
function start(found = documents(), failing = false, filled = "") {
  const box = new FakeBox(pageWords());
  box.field = filled;
  const source = new FakeDocuments(found, failing);
  const timer = new FakeTimer();
  startSearching(box, source, timer);
  const type = async (text: string): Promise<void> => {
    box.input(text);
    await settled();
    timer.pass();
  };
  return { box, source, timer, type };
}

test("words already in the field show the link and go with Enter, before the reader types", () => {
  const { box, source } = start(documents(), false, "limit");
  assert.equal(source.loads, 0);
  assert.equal(box.open, false);
  assert.equal(box.link?.address, "search.html?q=limit");
  box.enter();
  assert.equal(box.went, "search.html?q=limit");
});

test("the documents are fetched with the first letter typed, and only once", async () => {
  const { box, source, type } = start();
  assert.equal(source.loads, 0);
  box.input("w");
  assert.equal(source.loads, 1);
  await type("withdraw");
  await type("withdraw limit");
  assert.equal(source.loads, 1);
});

test("the status says search is loading until the documents are had", async () => {
  const box = new FakeBox(pageWords());
  let deliver: (found: readonly SearchDocument[]) => void = () => {};
  const source: Documents = { load: () => new Promise((resolve) => (deliver = resolve)) };
  const timer = new FakeTimer();
  startSearching(box, source, timer);
  box.input("withdraw");
  assert.equal(box.status, "Loading search.");
  timer.pass();
  deliver(documents());
  await settled();
  assert.equal(box.status, `Results: ${box.results.length}.`);
});

test("documents that cannot be fetched say so, and the next search tries again", async () => {
  const { box, source, type } = start(documents(), true);
  await type("withdraw");
  assert.equal(box.status, "Search could not be loaded.");
  assert.deepEqual(box.results, []);
  source.failing = false;
  await type("withdraws");
  assert.equal(source.loads, 2);
  assert.equal(box.status, `Results: ${box.results.length}.`);
});

test("documents that cannot be read say so, and the next search tries again", async () => {
  let loads = 0;
  const source: Documents = {
    load: () => {
      loads += 1;
      return Promise.resolve((loads === 1 ? {} : documents()) as unknown as SearchDocument[]);
    },
  };
  const box = new FakeBox(pageWords());
  const timer = new FakeTimer();
  startSearching(box, source, timer);
  box.input("withdraw");
  await settled();
  timer.pass();
  assert.equal(box.status, "Search could not be loaded.");
  box.input("withdraws");
  await settled();
  timer.pass();
  assert.equal(loads, 2);
  assert.equal(box.status, `Results: ${box.results.length}.`);
});

test("typing narrows the results at once, and the status waits for a pause of 300 milliseconds", async () => {
  const { box, timer, type } = start();
  await type("withdraw");
  const before = box.results.length;
  box.input("withdraw limit");
  await settled();
  assert.ok(box.results.length < before);
  assert.equal(timer.waited, 300);
  assert.equal(box.status, `Results: ${before}.`);
  timer.pass();
  assert.equal(box.status, `Results: ${box.results.length}.`);
});

test("results overtaken by more typing are not shown", async () => {
  const box = new FakeBox(pageWords());
  let deliver: (found: readonly SearchDocument[]) => void = () => {};
  const source: Documents = { load: () => new Promise((resolve) => (deliver = resolve)) };
  const timer = new FakeTimer();
  startSearching(box, source, timer);
  box.input("saque");
  box.input("limit refused");
  deliver(documents());
  await settled();
  assert.deepEqual(box.addresses(), ["banking/withdraw.html#withdraw-over-the-limit"]);
});

test("every word must match, by its start or one letter in five away", async () => {
  const { box, type } = start();
  await type("limit refused");
  assert.deepEqual(box.addresses(), ["banking/withdraw.html#withdraw-over-the-limit"]);
  await type("refus");
  assert.deepEqual(box.addresses(), ["banking/withdraw.html#withdraw-over-the-limit"]);
  await type("withdrawl refused");
  assert.deepEqual(box.addresses(), ["banking/withdraw.html#withdraw-over-the-limit"]);
  await type("poupança");
  assert.deepEqual(box.addresses(), ["saque.html"]);
  await type("zebra");
  assert.deepEqual(box.addresses(), []);
  assert.equal(box.status, "No results.");
});

test("tags and steps are searched, and a name counts most", async () => {
  const { box, type } = start();
  await type("@slow");
  assert.deepEqual(box.addresses(), ["banking/withdraw.html#withdraw-an-amount", "banking/withdraw.html#large-amounts"]);
  await type("balance");
  assert.equal(box.addresses()[0], "banking/withdraw.html#withdraw-within-the-balance");
  assert.equal(box.status, `Results: ${box.results.length}.`);
});

test("a background is found by its name and steps, and an examples block by its cells", async () => {
  const { box, type } = start();
  await type("limits in force");
  assert.deepEqual(box.addresses(), ["banking/withdraw.html#limits-in-force"]);
  await type("account");
  assert.deepEqual(box.addresses(), ["banking/withdraw.html#background"]);
  await type("emma");
  assert.deepEqual(box.addresses(), ["unnamed.html#examples-2"]);
});

test("doc strings and the tables of steps are not searched", async () => {
  const { box, type } = start();
  await type("dispensed");
  assert.deepEqual(box.addresses(), []);
  await type("banknote");
  assert.deepEqual(box.addresses(), []);
});

test("at most 50 results are listed, and the status says how many there are", async () => {
  const many = Array.from({ length: 60 }, (_, number) => ({ ...documents()[2]!, anchor: `withdraw-${number}` }));
  const { box, type } = start(many);
  await type("withdraw");
  assert.equal(box.results.length, 50);
  assert.equal(box.status, "Results: 60. The first 50 are listed.");
});

test("an empty search empties the results, the status and the link to every result", async () => {
  const { box, type } = start();
  await type("withdraw");
  assert.equal(box.link?.address, "search.html?q=withdraw");
  box.input("  ");
  assert.deepEqual(box.results, []);
  assert.equal(box.status, "");
  assert.equal(box.open, false);
  assert.equal(box.link, null);
});

test("the link to every result names the words without the spaces around them", async () => {
  const { box, type } = start();
  await type(" limit  refused ");
  assert.deepEqual(box.link?.label, [
    { text: "All results for \"", isName: false },
    { text: "limit  refused", isName: true },
    { text: "\"", isName: false },
  ]);
  assert.equal(box.link?.address, "search.html?q=limit++refused");
});

test("the list opens with results, and no result is chosen until an arrow chooses one", async () => {
  const { box, type } = start();
  await type("withdraw");
  assert.equal(box.open, true);
  assert.equal(box.chosen, null);
  box.down();
  assert.equal(box.chosen, 0);
  box.down();
  assert.equal(box.chosen, 1);
  box.up();
  box.up();
  assert.equal(box.chosen, null);
  box.up();
  assert.equal(box.chosen, box.results.length - 1);
  box.down();
  assert.equal(box.chosen, null);
  box.down();
  await type("withdraw limit");
  assert.equal(box.chosen, null);
});

test("Enter opens the chosen result, or the page of every result when none is chosen", async () => {
  const { box, type } = start();
  box.enter();
  assert.equal(box.went, null);
  await type("limit refused");
  box.enter();
  assert.equal(box.went, "search.html?q=limit+refused");
  box.down();
  box.enter();
  assert.equal(box.went, "banking/withdraw.html#withdraw-over-the-limit");
});

test("Enter before the documents are had opens the page of every result", () => {
  const box = new FakeBox(pageWords());
  startSearching(box, { load: () => new Promise(() => {}) }, new FakeTimer());
  box.input("withdraw");
  box.enter();
  assert.equal(box.went, "search.html?q=withdraw");
});

test("the pointer opens the result it picks", async () => {
  const { box, type } = start();
  await type("limit refused");
  box.pick(0);
  assert.equal(box.went, "banking/withdraw.html#withdraw-over-the-limit");
});

test("Escape closes the list and keeps the words, then a second Escape empties everything", async () => {
  const { box, type } = start();
  await type("withdraw");
  box.down();
  box.escape();
  assert.equal(box.open, false);
  assert.equal(box.chosen, null);
  assert.equal(box.field, "withdraw");
  assert.ok(box.results.length > 0);
  assert.notEqual(box.status, "");
  box.down();
  assert.equal(box.open, true);
  assert.equal(box.chosen, 0);
  box.escape();
  box.escape();
  assert.equal(box.field, "");
  assert.deepEqual(box.results, []);
  assert.equal(box.status, "");
  assert.equal(box.link, null);
});
