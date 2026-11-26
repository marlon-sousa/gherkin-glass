// Adapter: the address port over the window's location and history, catching the links that name an element of the page shown, or the address already shown, which change nothing by themselves.

import type { Address } from "../ports/canvas.ts";

export class DomAddress implements Address {
  private readonly window: Window;

  constructor(window: Window) {
    this.window = window;
  }

  hash(): string {
    return this.window.location.hash;
  }

  replace(hash: string): void {
    const address = new URL(this.window.location.href);
    address.hash = hash;
    this.window.history.replaceState(this.window.history.state, "", address.href);
  }

  onChange(handler: (hash: string) => void): void {
    this.window.addEventListener("hashchange", () => handler(this.window.location.hash));
    this.window.document.addEventListener("click", (event) => {
      const link = followedLink(event);
      if (link === null) {
        return;
      }
      const hash = link.getAttribute("href") ?? "";
      // A link to an element of the page shown changes no entry of the history.
      if (hash.startsWith("##")) {
        event.preventDefault();
        handler(hash);
      } else if (hash === this.window.location.hash || (hash === "#" && this.window.location.hash === "")) {
        handler(hash);
      }
    });
  }
}

/** The link of this shell a plain click follows, written from `#`; a click that opens another tab or window, or that a script stopped, follows none here. */
function followedLink(event: MouseEvent): HTMLAnchorElement | null {
  if (event.defaultPrevented || event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) {
    return null;
  }
  // The window's own classes are not the script's globals in a test, so the target is known by what it can do.
  const target = event.target as Element | null;
  const link = typeof target?.closest === "function" ? target.closest<HTMLAnchorElement>("a[href]") : null;
  return link !== null && (link.getAttribute("href") ?? "").startsWith("#") ? link : null;
}
