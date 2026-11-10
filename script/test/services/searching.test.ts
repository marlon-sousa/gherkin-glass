// The service of src/services/searching.ts, with fake ports, over tests/fixtures/search: docs/BEHAVIOUR.md, rules 152 to 154 and 156.

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
  focused = false;
  input: (text: string) => void = () => {};
  enter: () => void = () => {};
  escape: () => void = () => {};

  constructor(words: SearchWords) {
    this.given = words;
  }

  words(): SearchWords {
    return this.given;
  }
  onInput(handler: (text: string) => void): void {
    this.input = (text) => {
      this.field = text;
      handler(text);
    };
  }
  onEnter(handler: () => void): void {
    this.enter = handler;
  }
  onEscape(handler: () => void): void {
    this.escape = handler;
  }
  setStatus(text: string): void {
    this.status = text;
  }
  showResults(results: readonly Result[]): void {
    this.results = results;
  }
  clear(): void {
    this.field = "";
  }
  focusFirstResult(): void {
    this.focused = this.results.length > 0;
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

function start(found = documents(), failing = false) {
  const box = new FakeBox(pageWords());
  const source = new FakeDocuments(found, failing);
  const timer = new FakeTimer();
  startSearching(box, source, timer);
  const type = async (text: string): Promise<void> => {
    box.input(text);
    timer.pass();
    await settled();
  };
  return { box, source, timer, type };
}

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
  timer.pass();
  assert.equal(box.status, "Loading search.");
  deliver(documents());
  await settled();
  assert.equal(box.status, `Results: ${box.results.length}.`);
});

test("documents that cannot be fetched say so when a search runs, and the next search tries again", async () => {
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
  const box = new FakeBox(pageWords());
  let loads = 0;
  const source: Documents = {
    load: () => {
      loads += 1;
      return Promise.resolve((loads === 1 ? {} : documents()) as unknown as SearchDocument[]);
    },
  };
  const timer = new FakeTimer();
  startSearching(box, source, timer);
  box.input("withdraw");
  timer.pass();
  await settled();
  assert.equal(box.status, "Search could not be loaded.");
  box.input("withdraws");
  timer.pass();
  await settled();
  assert.equal(loads, 2);
  assert.equal(box.status, `Results: ${box.results.length}.`);
});

test("the focus is not moved by an Enter whose search was overtaken by more typing", async () => {
  const box = new FakeBox(pageWords());
  let deliver: (found: readonly SearchDocument[]) => void = () => {};
  const source: Documents = { load: () => new Promise((resolve) => (deliver = resolve)) };
  const timer = new FakeTimer();
  startSearching(box, source, timer);
  box.input("withdraw");
  box.enter();
  box.input("withdraw limit");
  timer.pass();
  deliver(documents());
  await settled();
  assert.ok(box.results.length > 0);
  assert.equal(box.focused, false);
});

test("nothing changes until the reader pauses for 300 milliseconds", () => {
  const { box, timer } = start();
  box.input("withdraw");
  assert.equal(timer.waited, 300);
  assert.equal(box.status, "");
  assert.deepEqual(box.results, []);
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
  assert.deepEqual(box.addresses(), ["banking/withdraw.html#withdraw-an-amount"]);
  await type("balance");
  assert.equal(box.addresses()[0], "banking/withdraw.html#withdraw-within-the-balance");
  assert.equal(box.status, `Results: ${box.results.length}.`);
});

test("doc strings and tables are not searched", async () => {
  const { box, type } = start();
  await type("dispensed");
  assert.deepEqual(box.addresses(), []);
  await type("banknote");
  assert.deepEqual(box.addresses(), []);
});

test("at most 50 results are listed, and the status says how many there are", async () => {
  const many = Array.from({ length: 60 }, (_, number) => ({ ...documents()[1]!, anchor: `withdraw-${number}` }));
  const { box, type } = start(many);
  await type("withdraw");
  assert.equal(box.results.length, 50);
  assert.equal(box.status, "Results: 60. The first 50 are listed.");
});

test("an empty search empties the results and the status", async () => {
  const { box, type } = start();
  await type("withdraw");
  box.input("  ");
  assert.deepEqual(box.results, []);
  assert.equal(box.status, "");
});

test("Enter moves to the first result, and Escape empties everything", async () => {
  const { box, type } = start();
  box.input("zebra");
  box.enter();
  await settled();
  assert.equal(box.focused, false);
  await type("withdraw");
  box.enter();
  await settled();
  assert.equal(box.focused, true);
  box.escape();
  assert.equal(box.field, "");
  assert.deepEqual(box.results, []);
  assert.equal(box.status, "");
});
