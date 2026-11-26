// Port: the shell a page is drawn in, its `main`, its title, its breadcrumb, its live region and where the focus is, and the reader's moves between addresses.

import type { PageFrame } from "../policies/page_frame.ts";

export interface Canvas {
  /** Marks the `main` as being drawn, or drawn, with `aria-busy`. */
  setBusy(busy: boolean): void;
  focusMain(): void;
  setFrame(frame: PageFrame): void;
  setTitle(title: string): void;
  /** Says `words` in the live region. */
  announce(words: string): void;
  /** Moves the focus to the page's `h1` and scrolls it into view. */
  focusHeading(): void;
  /** Moves the focus to the element whose `id` is `id` and scrolls it into view; false when the page has none. */
  focusElement(id: string): boolean;
  /** Puts back the front page the shell's `main` arrived holding, and returns the title the shell arrived with; null for a shell that holds none. */
  restoreFrontPage(): string | null;
  /** Draws in the `main` the paragraph that says the page could not be loaded. */
  drawFailure(words: string): void;
}

/** The address bar and the history: what follows the `#`, and its changes. */
export interface Address {
  hash(): string;
  /** Puts `hash` back in the address bar without a new entry of the history, and without drawing. */
  replace(hash: string): void;
  /** The address changed to `hash`, by a link, by the history or in the address bar, or a link to `hash` was followed while it was the address already. */
  onChange(handler: (hash: string) => void): void;
}
