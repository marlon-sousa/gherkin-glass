// The service of src/services/collapsing.ts, with fake ports: docs/BEHAVIOUR.md, rules 51 to 55 and 97 to 99.

import assert from "node:assert/strict";
import { test } from "node:test";

import { startCollapsing } from "../../src/services/collapsing.ts";
import { FORGETFUL, FakeBackgrounds, FakeBlock, FakeMemory, FakePage } from "../support/page_fakes.ts";

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
  startCollapsing(page, new FakeMemory(), PATH, null);
  assert.deepEqual(states(page), [false, false, false, true]);
});

test("pressing a button switches its block, and is remembered", () => {
  const page = new FakePage(blocks());
  const memory = new FakeMemory();
  startCollapsing(page, memory, PATH, null);
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
  startCollapsing(page, memory, PATH, null);
  assert.deepEqual(states(page), [false, false, true, true]);
});

test("collapse all and expand all set every block, and are remembered as each block's state", () => {
  const page = new FakePage(blocks());
  const memory = new FakeMemory();
  startCollapsing(page, memory, PATH, null);
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
  startCollapsing(page, memory, PATH, null);
  assert.deepEqual(states(page), [false, true, true, true]);
  assert.equal(memory.get(key("refunds-after-thirty-days")), "expanded");
});

test("following a link on the page expands the block it points to and the blocks around it", () => {
  const page = new FakePage(blocks());
  startCollapsing(page, new FakeMemory(), PATH, null);
  page.collapseAll();
  page.follow("cheap");
  assert.deepEqual(states(page), [false, true, true, true]);
  page.follow("no-such-anchor");
  assert.deepEqual(states(page), [false, true, true, true]);
});

test("a memory that keeps nothing leaves the page working", () => {
  const page = new FakePage(blocks());
  startCollapsing(page, FORGETFUL, PATH, null);
  page.toggle(page.found[0]!);
  assert.equal(page.found[0]!.isExpanded(), false);
});

// The backgrounds' forms, rules 97 to 99, and the state of the backgrounds whose steps a scenario starts with, rules 51, 54 and 98.

const SITE = "/site/";
const FORM = "gherkin-glass:/site/:backgrounds";
const inlineKey = (anchor: string): string => `gherkin-glass:inline:${PATH}#${anchor}`;

/** A feature's background whose steps its scenario starts with, collapsed as a page written inline writes it, and a rule's background no scenario starts with. */
function withBackgrounds(written: boolean): FakeBlock[] {
  return [new FakeBlock("background", null, !written, true), new FakeBlock("scenario", null), new FakeBlock("rule-background", null)];
}

function startForms(written: boolean, memory: FakeMemory | typeof FORGETFUL = new FakeMemory()): [FakePage, FakeBackgrounds] {
  const page = new FakePage(withBackgrounds(written));
  const backgrounds = new FakeBackgrounds(written);
  startCollapsing(page, memory, PATH, { backgrounds, siteRoot: SITE });
  return [page, backgrounds];
}

test("the page keeps the form it was written in when nothing is remembered, with the backgrounds in its state", () => {
  const [inline, inlineForm] = startForms(true);
  assert.equal(inlineForm.inline, true);
  assert.deepEqual(states(inline), [false, true, true]);
  const [linked, linkedForm] = startForms(false);
  assert.equal(linkedForm.inline, false);
  assert.deepEqual(states(linked), [true, true, true]);
});

test("a remembered form wins over the form the page was written in, and brings its backgrounds' state", () => {
  const linkedMemory = new FakeMemory();
  linkedMemory.set(FORM, "linked");
  const [linked, linkedForm] = startForms(true, linkedMemory);
  assert.equal(linkedForm.inline, false);
  assert.deepEqual(states(linked), [true, true, true]);
  const inlineMemory = new FakeMemory();
  inlineMemory.set(FORM, "inline");
  const [inline, inlineForm] = startForms(false, inlineMemory);
  assert.equal(inlineForm.inline, true);
  assert.deepEqual(states(inline), [false, true, true]);
});

test("switching shows the other form for the site, and collapses or expands every background whose steps it moves", () => {
  const memory = new FakeMemory();
  const [page, backgrounds] = startForms(true, memory);
  page.toggle(page.found[2]!);
  backgrounds.switched(false);
  assert.equal(backgrounds.inline, false);
  assert.equal(memory.get(FORM), "linked");
  assert.deepEqual(states(page), [true, true, false]);
  assert.equal(memory.get(key("background")), "expanded");
  backgrounds.switched(true);
  assert.equal(backgrounds.inline, true);
  assert.equal(memory.get(FORM), "inline");
  assert.deepEqual(states(page), [false, true, false]);
  assert.equal(memory.get(inlineKey("background")), "collapsed");
});

test("a background's state is remembered for each form apart", () => {
  const memory = new FakeMemory();
  const [page, backgrounds] = startForms(true, memory);
  const background = page.found[0]!;
  page.toggle(background);
  assert.equal(memory.get(inlineKey("background")), "expanded");
  backgrounds.switched(false);
  page.toggle(background);
  assert.equal(memory.get(key("background")), "collapsed");
  page.collapseAll();
  page.expandAll();
  assert.equal(memory.get(inlineKey("background")), "expanded");
  memory.set(key("background"), "collapsed");
  const [again] = startForms(false, memory);
  assert.equal(again.found[0]!.isExpanded(), false);
  memory.set(FORM, "inline");
  const [inline] = startForms(false, memory);
  assert.equal(inline.found[0]!.isExpanded(), true);
});

test("a page shown again as it was left takes the form another page chose meanwhile, with its backgrounds' state", () => {
  const memory = new FakeMemory();
  const [page, backgrounds] = startForms(true, memory);
  backgrounds.switched(false);
  memory.set(FORM, "inline");
  backgrounds.returned();
  assert.equal(backgrounds.inline, true);
  assert.deepEqual(states(page), [false, true, true]);
});

test("a memory that keeps nothing leaves the page as it was written, and switching still works", () => {
  const [page, backgrounds] = startForms(false, FORGETFUL);
  assert.equal(backgrounds.inline, false);
  backgrounds.switched(true);
  assert.equal(backgrounds.inline, true);
  assert.deepEqual(states(page), [false, true, true]);
  backgrounds.returned();
  assert.equal(backgrounds.inline, false);
  assert.deepEqual(states(page), [true, true, true]);
});
