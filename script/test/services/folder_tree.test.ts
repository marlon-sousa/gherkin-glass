// The service of src/services/folder_tree.ts, with fake ports: docs/BEHAVIOUR.md, rules 87, 88, 90 and 93.

import assert from "node:assert/strict";
import { test } from "node:test";

import { buildTree, startTreeControls } from "../../src/services/folder_tree.ts";
import type { Memory } from "../../src/ports/memory.ts";
import type { FocusPlace, TreeItem, TreeSwitch, TreeView } from "../../src/ports/tree.ts";
import type { WindowWidth } from "../../src/ports/width.ts";

const ITEMS: TreeItem[] = [
  { text: "Home", address: "index.html" },
  { text: "a", address: "a/index.html", items: [{ text: "F", address: "a/f.html", items: [{ text: "S", address: "a/f.html#s" }] }] },
  { text: "b", address: "b/index.html", items: [{ text: "G", address: "b/g.html" }] },
];

class FakeSwitch implements TreeSwitch {
  checked: boolean;
  open = false;
  shown: boolean;
  disabled = false;
  focused: "button" | "checkbox" | null = null;
  focusAt: FocusPlace = null;
  change: (checked: boolean) => void = () => {};
  press: () => void = () => {};
  escape: () => void = () => {};
  constructor(configured: boolean) {
    this.checked = configured;
    this.shown = configured;
  }
  configured(): boolean {
    return this.checked;
  }
  setChecked(checked: boolean): void {
    this.checked = checked;
  }
  setOpen(open: boolean): void {
    this.open = open;
  }
  setShown(shown: boolean): void {
    this.shown = shown;
  }
  disable(): void {
    this.disabled = true;
  }
  onChange(handler: (checked: boolean) => void): void {
    this.change = handler;
  }
  onPress(handler: () => void): void {
    this.press = handler;
  }
  onEscape(handler: () => void): void {
    this.escape = handler;
  }
  focus(): FocusPlace {
    return this.focusAt;
  }
  focusButton(): void {
    this.focused = "button";
  }
  focusCheckbox(): void {
    this.focused = "checkbox";
  }
}

class FakeWidth implements WindowWidth {
  change: (narrow: boolean) => void = () => {};
  constructor(public isNarrow: boolean) {}
  narrow(): boolean {
    return this.isNarrow;
  }
  onChange(handler: (narrow: boolean) => void): void {
    this.change = handler;
  }
  resize(narrow: boolean): void {
    this.isNarrow = narrow;
    this.change(narrow);
  }
}

class FakeView implements TreeView {
  current: string | null = null;
  expanded: string[] = [];
  toggle: (address: string, expanded: boolean) => void = () => {};
  build(items: readonly TreeItem[], current: string | null, expanded: (address: string) => boolean): void {
    this.current = current;
    const walk = (list: readonly TreeItem[]): void => {
      for (const item of list) {
        if (expanded(item.address)) {
          this.expanded.push(item.address);
        }
        walk(item.items ?? []);
      }
    };
    walk(items);
  }
  onToggle(handler: (address: string, expanded: boolean) => void): void {
    this.toggle = handler;
  }
}

/** The tree as a shell starts it: its controls, then the tree of `items` with `path` expanded, or, with no items, its controls disabled. */
function startTree(items: readonly TreeItem[] | null, toggle: TreeSwitch, view: TreeView, width: WindowWidth, memory: Memory, sitePath: string, path: readonly string[]): void {
  const controls = startTreeControls(toggle, width, memory, sitePath);
  if (items === null) {
    controls.disable();
  } else {
    buildTree(view, memory, sitePath, items, path);
  }
}

function memory(kept: Map<string, string>): Memory {
  return { get: (key) => kept.get(key) ?? null, set: (key, value) => void kept.set(key, value) };
}

test("the path to the current page is expanded, with remembered items, and every choice is kept for the site", () => {
  const kept = new Map([
    ["gherkin-glass:/site/ tree a/index.html", "collapsed"],
    ["gherkin-glass:/site/ tree b/index.html", "expanded"],
  ]);
  const toggle = new FakeSwitch(true);
  const view = new FakeView();
  startTree(ITEMS, toggle, view, new FakeWidth(false), memory(kept), "/site/", ["a/f.html", "a/index.html"]);
  assert.equal(view.current, "a/f.html");
  assert.deepEqual(view.expanded, ["a/index.html", "a/f.html", "b/index.html"]);
  assert.equal(toggle.shown, true);
  view.toggle("b/index.html", false);
  assert.equal(kept.get("gherkin-glass:/site/ tree b/index.html"), "collapsed");
  toggle.change(false);
  assert.equal(toggle.shown, false);
  assert.equal(kept.get("gherkin-glass:/site/ tree"), "hidden");
});

test("a narrow window starts with the tree hidden, whatever the checkbox says, and its button shows and hides it", () => {
  const kept = new Map([["gherkin-glass:/ tree", "shown"]]);
  const toggle = new FakeSwitch(true);
  startTree(ITEMS, toggle, new FakeView(), new FakeWidth(true), memory(kept), "/", ["index.html"]);
  assert.deepEqual([toggle.checked, toggle.open, toggle.shown], [true, false, false]);
  toggle.press();
  assert.deepEqual([toggle.open, toggle.shown], [true, true]);
  toggle.press();
  assert.deepEqual([toggle.open, toggle.shown], [false, false]);
  // The button's state is never kept, and leaves the checkbox's choice as it was.
  assert.deepEqual([...kept], [["gherkin-glass:/ tree", "shown"]]);
});

test("Escape in the tree on a narrow window hides it and puts the focus on the button", () => {
  const toggle = new FakeSwitch(true);
  startTree(ITEMS, toggle, new FakeView(), new FakeWidth(true), memory(new Map()), "/", ["index.html"]);
  toggle.escape();
  assert.equal(toggle.focused, null);
  toggle.press();
  toggle.escape();
  assert.deepEqual([toggle.open, toggle.shown, toggle.focused], [false, false, "button"]);
});

test("Escape in the tree on a wide window does nothing", () => {
  const toggle = new FakeSwitch(true);
  startTree(ITEMS, toggle, new FakeView(), new FakeWidth(false), memory(new Map()), "/", ["index.html"]);
  toggle.escape();
  assert.deepEqual([toggle.shown, toggle.focused], [true, null]);
});

test("resizing across the narrow width switches between the checkbox and the button, both ways", () => {
  const toggle = new FakeSwitch(true);
  const width = new FakeWidth(false);
  startTree(ITEMS, toggle, new FakeView(), width, memory(new Map()), "/", ["index.html"]);
  assert.equal(toggle.shown, true);
  width.resize(true);
  assert.deepEqual([toggle.open, toggle.shown], [false, false]);
  toggle.press();
  assert.equal(toggle.shown, true);
  width.resize(false);
  assert.deepEqual([toggle.open, toggle.shown], [false, true]);
  toggle.change(false);
  width.resize(true);
  toggle.press();
  assert.equal(toggle.shown, true);
  width.resize(false);
  assert.equal(toggle.shown, false);
});

test("a resize that hides the tree with the focus inside it moves the focus to the control it shows", () => {
  const toggle = new FakeSwitch(true);
  const width = new FakeWidth(false);
  startTree(ITEMS, toggle, new FakeView(), width, memory(new Map()), "/", ["index.html"]);
  toggle.focusAt = "tree";
  width.resize(true);
  assert.equal(toggle.focused, "button");
  toggle.focused = null;
  toggle.change(false);
  toggle.press();
  width.resize(false);
  assert.equal(toggle.focused, "checkbox");
});

test("a resize that hides the control with the focus moves the focus to the one it shows in its place", () => {
  const toggle = new FakeSwitch(true);
  const width = new FakeWidth(false);
  startTree(ITEMS, toggle, new FakeView(), width, memory(new Map()), "/", ["index.html"]);
  toggle.focusAt = "checkbox";
  width.resize(true);
  assert.equal(toggle.focused, "button");
  toggle.focused = null;
  toggle.focusAt = "button";
  width.resize(false);
  assert.equal(toggle.focused, "checkbox");
});

test("a resize that keeps the tree shown, or hides it with the focus elsewhere, leaves the focus where it is", () => {
  const toggle = new FakeSwitch(true);
  const width = new FakeWidth(true);
  startTree(ITEMS, toggle, new FakeView(), width, memory(new Map()), "/", ["index.html"]);
  toggle.press();
  toggle.focusAt = "tree";
  width.resize(false);
  assert.deepEqual([toggle.shown, toggle.focused], [true, null]);
  toggle.focusAt = null;
  width.resize(true);
  assert.deepEqual([toggle.shown, toggle.focused], [false, null]);
});

test("data that cannot be loaded disables the checkbox and the button and builds nothing", () => {
  const toggle = new FakeSwitch(true);
  const view = new FakeView();
  const width = new FakeWidth(false);
  startTree(null, toggle, view, width, memory(new Map()), "/", []);
  assert.equal(toggle.disabled, true);
  assert.deepEqual([toggle.checked, toggle.shown], [false, false]);
  assert.deepEqual(view.expanded, []);
  width.resize(true);
  toggle.press();
  assert.deepEqual([toggle.open, toggle.shown], [false, false]);
});
