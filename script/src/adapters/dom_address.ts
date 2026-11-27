// Adapter: the address port over the window's location and history, catching the links that name an element of the page shown, or the address already shown, which change nothing by themselves.

import { followedLink } from "./followed_link.ts";
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
      const hash = followedLink(event)?.getAttribute("href") ?? "";
      if (!hash.startsWith("#")) {
        return;
      }
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
