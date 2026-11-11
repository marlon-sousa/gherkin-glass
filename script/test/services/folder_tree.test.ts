// The service of src/services/folder_tree.ts, with fake ports: docs/BEHAVIOUR.md, rules 87, 88, 90 and 93.

import assert from "node:assert/strict";
import { test } from "node:test";

import { startFolderTree } from "../../src/services/folder_tree.ts";
import type { Memory } from "../../src/ports/memory.ts";
import type { TreeItem, TreeSource, TreeSwitch, TreeView } from "../../src/ports/tree.ts";

const ITEMS: TreeItem[] = [
  { text: "Home", address: "index.html" },
  { text: "a", address: "a/index.html", items: [{ text: "F", address: "a/f.html", items: [{ text: "S", address: "a/f.html#s" }] }] },
  { text: "b", address: "b/index.html", items: [{ text: "G", address: "b/g.html" }] },
];

class FakeSwitch implements TreeSwitch {
  shown: boolean;
  disabled = false;
  change: (shown: boolean) => void = () => {};
  constructor(configured: boolean) {
    this.shown = configured;
  }
  configured(): boolean {
    return this.shown;
  }
  setShown(shown: boolean): void {
    this.shown = shown;
  }
  disable(): void {
    this.shown = false;
    this.disabled = true;
  }
  onChange(handler: (shown: boolean) => void): void {
    this.change = handler;
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

const source = (items: readonly TreeItem[] | null): TreeSource => ({ load: () => Promise.resolve(items) });

function memory(kept: Map<string, string>): Memory {
  return { get: (key) => kept.get(key) ?? null, set: (key, value) => void kept.set(key, value) };
}

test("the path to the current page is expanded, with remembered items, and every choice is kept for the site", async () => {
  const kept = new Map([
    ["gherkin-glass:/site/ tree a/index.html", "collapsed"],
    ["gherkin-glass:/site/ tree b/index.html", "expanded"],
  ]);
  const toggle = new FakeSwitch(true);
  const view = new FakeView();
  await startFolderTree(source(ITEMS), toggle, view, memory(kept), { sitePath: "/site/", page: "a/f.html", narrow: false });
  assert.equal(view.current, "a/f.html");
  assert.deepEqual(view.expanded, ["a/index.html", "a/f.html", "b/index.html"]);
  assert.equal(toggle.shown, true);
  view.toggle("b/index.html", false);
  assert.equal(kept.get("gherkin-glass:/site/ tree b/index.html"), "collapsed");
  toggle.change(false);
  assert.equal(toggle.shown, false);
  assert.equal(kept.get("gherkin-glass:/site/ tree"), "hidden");
});

test("a narrow window hides the tree unless the reader chose to show it", async () => {
  const narrow = new FakeSwitch(true);
  await startFolderTree(source(ITEMS), narrow, new FakeView(), memory(new Map()), { sitePath: "/", page: "index.html", narrow: true });
  assert.equal(narrow.shown, false);
  const chosen = new FakeSwitch(false);
  await startFolderTree(source(ITEMS), chosen, new FakeView(), memory(new Map([["gherkin-glass:/ tree", "shown"]])), { sitePath: "/", page: "index.html", narrow: true });
  assert.equal(chosen.shown, true);
});

test("data that cannot be loaded disables the checkbox and builds nothing", async () => {
  const toggle = new FakeSwitch(true);
  const view = new FakeView();
  await startFolderTree(source(null), toggle, view, memory(new Map()), { sitePath: "/", page: null, narrow: false });
  assert.equal(toggle.disabled, true);
  assert.equal(toggle.shown, false);
  assert.deepEqual(view.expanded, []);
});
