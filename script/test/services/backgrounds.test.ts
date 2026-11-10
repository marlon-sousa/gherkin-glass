// The service of src/services/backgrounds.ts, with fake ports: docs/BEHAVIOUR.md, "Backgrounds".

import assert from "node:assert/strict";
import { test } from "node:test";

import { startBackgrounds } from "../../src/services/backgrounds.ts";
import type { Backgrounds } from "../../src/ports/backgrounds.ts";
import type { Memory } from "../../src/ports/memory.ts";

class FakeBackgrounds implements Backgrounds {
  readonly written: boolean;
  inline: boolean;
  switched: (inline: boolean) => void = () => {};
  returned: () => void = () => {};

  constructor(written: boolean) {
    this.written = written;
    this.inline = written;
  }

  writtenInline(): boolean {
    return this.written;
  }

  setInline(inline: boolean): void {
    this.inline = inline;
  }

  onSwitch(handler: (inline: boolean) => void): void {
    this.switched = handler;
  }

  onReturn(handler: () => void): void {
    this.returned = handler;
  }
}

function memory(kept: Record<string, string> = {}): Memory & { kept: Record<string, string> } {
  return { kept, get: (key) => kept[key] ?? null, set: (key, value) => void (kept[key] = value) };
}

const KEY = "gherkin-glass:/site/:backgrounds";

test("the page keeps the form it was written in when nothing is remembered", () => {
  for (const written of [true, false]) {
    const backgrounds = new FakeBackgrounds(written);
    startBackgrounds(backgrounds, memory(), "/site/");
    assert.equal(backgrounds.inline, written);
  }
});

test("a remembered form wins over the form the page was written in", () => {
  const linked = new FakeBackgrounds(true);
  startBackgrounds(linked, memory({ [KEY]: "linked" }), "/site/");
  assert.equal(linked.inline, false);
  const inline = new FakeBackgrounds(false);
  startBackgrounds(inline, memory({ [KEY]: "inline" }), "/site/");
  assert.equal(inline.inline, true);
});

test("switching shows the other form and is remembered for the site", () => {
  const backgrounds = new FakeBackgrounds(true);
  const kept = memory();
  startBackgrounds(backgrounds, kept, "/site/");
  backgrounds.switched(false);
  assert.equal(backgrounds.inline, false);
  assert.deepEqual(kept.kept, { [KEY]: "linked" });
  backgrounds.switched(true);
  assert.equal(backgrounds.inline, true);
  assert.deepEqual(kept.kept, { [KEY]: "inline" });
});

test("a page shown again as it was left takes the choice another page made meanwhile", () => {
  const backgrounds = new FakeBackgrounds(true);
  const kept = memory();
  startBackgrounds(backgrounds, kept, "/site/");
  backgrounds.switched(false);
  kept.kept[KEY] = "inline";
  backgrounds.returned();
  assert.equal(backgrounds.inline, true);
});

test("a memory that keeps nothing leaves the page as it was written, and switching still works", () => {
  const forgetful: Memory = { get: () => null, set: () => {} };
  const backgrounds = new FakeBackgrounds(false);
  startBackgrounds(backgrounds, forgetful, "/site/");
  assert.equal(backgrounds.inline, false);
  backgrounds.switched(true);
  assert.equal(backgrounds.inline, true);
});
