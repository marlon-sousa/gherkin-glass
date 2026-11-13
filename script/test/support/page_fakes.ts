// Fake ports of the page, its backgrounds' checkbox and the browser's memory, for the tests of src/services/collapsing.ts, kept apart so that the test file stays short.

import type { Backgrounds } from "../../src/ports/backgrounds.ts";
import type { Memory } from "../../src/ports/memory.ts";
import type { Block, Page } from "../../src/ports/page.ts";

export class FakeBlock implements Block {
  readonly anchor: string;
  readonly parent: Block | null;
  readonly expandedWhenUnchecked: boolean | null;
  readonly inScenario: boolean;
  private expanded: boolean;

  constructor(anchor: string, parent: Block | null, expanded = true, expandedWhenUnchecked: boolean | null = null, inScenario = false) {
    this.anchor = anchor;
    this.parent = parent;
    this.expanded = expanded;
    this.expandedWhenUnchecked = expandedWhenUnchecked;
    this.inScenario = inScenario;
  }

  isExpanded(): boolean {
    return this.expanded;
  }

  setExpanded(expanded: boolean): void {
    this.expanded = expanded;
  }
}

export class FakePage implements Page {
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

export class FakeBackgrounds implements Backgrounds {
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

export class FakeMemory implements Memory {
  readonly kept = new Map<string, string>();

  get(key: string): string | null {
    return this.kept.get(key) ?? null;
  }

  set(key: string, value: string): void {
    this.kept.set(key, value);
  }
}

/** A memory that keeps nothing, as a browser whose storage is forbidden. */
export const FORGETFUL: Memory = { get: () => null, set: () => {} };
