// Adapter: the window width port over a media query that matches a narrow window.

import type { WindowWidth } from "../ports/width.ts";

export class MediaWidth implements WindowWidth {
  constructor(private readonly query: MediaQueryList) {}

  narrow(): boolean {
    return this.query.matches;
  }

  onChange(handler: (narrow: boolean) => void): void {
    this.query.addEventListener("change", (event) => handler(event.matches));
  }
}
