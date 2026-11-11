// Adapter: the memory port over the browser's local storage.

import type { Memory } from "../ports/memory.ts";

/** A browser that forbids storage, as some private windows do, throws on every use. */
export class LocalMemory implements Memory {
  get(key: string): string | null {
    try {
      return window.localStorage.getItem(key);
    } catch {
      return null;
    }
  }

  set(key: string, value: string): void {
    try {
      window.localStorage.setItem(key, value);
    } catch {
      // Nothing is kept, and the page works the same.
    }
  }
}
