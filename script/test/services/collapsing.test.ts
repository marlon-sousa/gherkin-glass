// The service of src/services/collapsing.ts, with fake ports shaped as
// tests/fixtures/pages/expected/billing/refunds.html is: docs/BEHAVIOUR.md, rules 51 to 55.

import assert from "node:assert/strict";
import { test } from "node:test";

import { startCollapsing } from "../../src/services/collapsing.ts";
import type { Memory } from "../../src/ports/memory.ts";
import type { Block, Page } from "../../src/ports/page.ts";

class FakeBlock implements Block {
  readonly anchor: string;
  readonly parent: Block | null;
  private expanded: boolean;

  constructor(anchor: string, parent: Block | null, expanded = true) {
    this.anchor = anchor;
    this.parent = parent;
    this.expanded = expanded;
  }

  isExpanded(): boolean {
    return this.expanded;
  }

  setExpanded(expanded: boolean): void {
    this.expanded = expanded;
  }
}

class FakePage implements Page {
  readonly found: FakeBlock[];
  readonly opened: string | null;
  toggle: (block: Block) => void = () => {};
  collapseAll: () => void = () => {};
  expandAll: () => void = () => {};
  follow: (anchor: string) => void = () => {};

  constructor(found: FakeBlock[], opened: string | null = null) {
    this.found = found;
    this.opened = opened;
  }

  blocks(): readonly Block[] {
    return this.found;
  }

  target(): string | null {
    return this.opened;
  }

  onToggle(handler: (block: Block) => void): void {
    this.toggle = handler;
  }

  onCollapseAll(handler: () => void): void {
    this.collapseAll = handler;
  }

  onExpandAll(handler: () => void): void {
    this.expandAll = handler;
  }

  onFollow(handler: (anchor: string) => void): void {
    this.follow = handler;
  }
}

class FakeMemory implements Memory {
  readonly kept = new Map<string, string>();

  get(key: string): string | null {
    return this.kept.get(key) ?? null;
  }

  set(key: string, value: string): void {
    this.kept.set(key, value);
  }
}

const PATH = "/billing/refunds.html";
const key = (anchor: string): string => `gherkin-glass:${PATH}#${anchor}`;

/** A rule holding a scenario holding an examples block, beside a background, each in the given state. */
function blocks(expanded = true): FakeBlock[] {
  const background = new FakeBlock("background", null, expanded);
  const rule = new FakeBlock("refunds-after-thirty-days", null, expanded);
  const scenario = new FakeBlock("store-credit", rule, expanded);
  const examples = new FakeBlock("cheap", scenario, true);
  return [background, rule, scenario, examples];
}

const states = (page: FakePage): boolean[] => page.found.map((block) => block.isExpanded());

test("the configured state stands until the reader chooses otherwise", () => {
  const page = new FakePage(blocks(false));
  startCollapsing(page, new FakeMemory(), PATH);
  assert.deepEqual(states(page), [false, false, false, true]);
});

test("pressing a button switches its block, and is remembered", () => {
  const page = new FakePage(blocks());
  const memory = new FakeMemory();
  startCollapsing(page, memory, PATH);
  const rule = page.found[1]!;
  page.toggle(rule);
  assert.deepEqual(states(page), [true, false, true, true]);
  assert.equal(memory.get(key("refunds-after-thirty-days")), "collapsed");
  page.toggle(rule);
  assert.equal(rule.isExpanded(), true);
  assert.equal(memory.get(key("refunds-after-thirty-days")), "expanded");
});

test("a remembered state wins over the configured one when the page loads again", () => {
  const memory = new FakeMemory();
  memory.set(key("background"), "collapsed");
  memory.set(key("store-credit"), "expanded");
  const page = new FakePage(blocks(false).map((block, n) => (n === 0 ? new FakeBlock("background", null, true) : block)));
  startCollapsing(page, memory, PATH);
  assert.deepEqual(states(page), [false, false, true, true]);
});

test("collapse all and expand all set every block, and are remembered as each block's state", () => {
  const page = new FakePage(blocks());
  const memory = new FakeMemory();
  startCollapsing(page, memory, PATH);
  page.collapseAll();
  assert.deepEqual(states(page), [false, false, false, false]);
  assert.equal(memory.get(key("cheap")), "collapsed");
  page.expandAll();
  assert.deepEqual(states(page), [true, true, true, true]);
  assert.equal(memory.get(key("background")), "expanded");
});

test("opening the page at an anchor expands its block and the blocks around it", () => {
  const page = new FakePage(blocks(false), "store-credit");
  const memory = new FakeMemory();
  startCollapsing(page, memory, PATH);
  assert.deepEqual(states(page), [false, true, true, true]);
  assert.equal(memory.get(key("refunds-after-thirty-days")), "expanded");
});

test("following a link on the page expands the block it points to and the blocks around it", () => {
  const page = new FakePage(blocks());
  startCollapsing(page, new FakeMemory(), PATH);
  page.collapseAll();
  page.follow("cheap");
  assert.deepEqual(states(page), [false, true, true, true]);
  page.follow("no-such-anchor");
  assert.deepEqual(states(page), [false, true, true, true]);
});

test("a memory that keeps nothing leaves the page working", () => {
  const page = new FakePage(blocks());
  const forgetful: Memory = { get: () => null, set: () => {} };
  startCollapsing(page, forgetful, PATH);
  page.toggle(page.found[0]!);
  assert.equal(page.found[0]!.isExpanded(), false);
});
