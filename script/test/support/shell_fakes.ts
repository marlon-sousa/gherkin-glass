// Fake ports of a shell, its address and a timer, for the tests of src/services/navigation.ts, each recording what is done to it in order.

import type { PageFrame } from "../../src/policies/page_frame.ts";
import type { Address, Canvas } from "../../src/ports/canvas.ts";
import type { Timer } from "../../src/ports/timer.ts";

export class FakeCanvas implements Canvas {
  readonly done: string[] = [];
  /** The `id`s the page drawn has. */
  ids = new Set<string>();
  frontPage: string | null = null;

  setBusy(busy: boolean): void {
    this.done.push(`busy ${busy}`);
  }

  focusMain(): void {
    this.done.push("focus main");
  }

  setFrame(frame: PageFrame): void {
    this.done.push(`frame ${frame.lang}`);
  }

  setTitle(title: string): void {
    this.done.push(`title ${title}`);
  }

  announce(words: string): void {
    this.done.push(`announce ${words}`);
  }

  focusHeading(): void {
    this.done.push("focus h1");
  }

  focusElement(id: string): boolean {
    if (!this.ids.has(id)) {
      return false;
    }
    this.done.push(`focus ${id}`);
    return true;
  }

  holdsFrontPage(): boolean {
    return this.frontPage !== null;
  }

  restoreFrontPage(): string | null {
    return this.frontPage;
  }

  drawFailure(words: string): void {
    this.done.push(`failure ${words}`);
  }
}

export class FakeAddress implements Address {
  readonly replaced: string[] = [];
  private handler: (hash: string) => void = () => {};

  constructor(public current: string) {}

  hash(): string {
    return this.current;
  }

  replace(hash: string): void {
    this.replaced.push(hash);
    this.current = hash;
  }

  onChange(handler: (hash: string) => void): void {
    this.handler = handler;
  }

  /** The reader goes to `hash`. */
  go(hash: string): void {
    this.current = hash;
    this.handler(hash);
  }
}

/** Runs what is waited for only when the test says the time has passed. */
export class FakeTimer implements Timer {
  waits: number[] = [];
  private pending: (() => void) | null = null;

  after(milliseconds: number, handler: () => void): void {
    this.waits.push(milliseconds);
    this.pending = handler;
  }

  cancel(): void {
    this.pending = null;
  }

  pass(): void {
    const pending = this.pending;
    this.pending = null;
    pending?.();
  }
}

/** Lets the promises already resolved run their continuations. */
export function settled(): Promise<void> {
  return new Promise((done) => setImmediate(done));
}
