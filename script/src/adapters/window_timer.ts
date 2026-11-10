// Adapter: the timer port over the browser's setTimeout.

import type { Timer } from "../ports/timer.ts";

export class WindowTimer implements Timer {
  private pending: ReturnType<typeof setTimeout> | null = null;

  after(milliseconds: number, handler: () => void): void {
    this.cancel();
    this.pending = setTimeout(() => {
      this.pending = null;
      handler();
    }, milliseconds);
  }

  cancel(): void {
    if (this.pending !== null) {
      clearTimeout(this.pending);
      this.pending = null;
    }
  }
}
