// Adapter: the page port over the page's markup, as templates/block.html, templates/feature.html and templates/folder.html write it.

import type { Block, Page } from "../ports/page.ts";

class DomBlock implements Block {
  readonly anchor: string;
  readonly parent: Block | null;
  readonly button: HTMLButtonElement;
  private readonly content: HTMLElement;

  constructor(anchor: string, button: HTMLButtonElement, content: HTMLElement, parent: Block | null) {
    this.anchor = anchor;
    this.button = button;
    this.content = content;
    this.parent = parent;
  }

  isExpanded(): boolean {
    return this.button.getAttribute("aria-expanded") === "true";
  }

  setExpanded(expanded: boolean): void {
    this.button.setAttribute("aria-expanded", String(expanded));
    this.content.hidden = !expanded;
  }
}

export class DomPage implements Page {
  private readonly found: DomBlock[] = [];
  private readonly document: Document;

  constructor(document: Document) {
    this.document = document;
    const byElement = new Map<Element, DomBlock>();
    for (const element of document.querySelectorAll(".ghg-block")) {
      const heading = element.querySelector<HTMLElement>(":scope > :is(h2, h3, h4)");
      const button = heading?.querySelector<HTMLButtonElement>(":scope > button.ghg-toggle");
      const content = element.querySelector<HTMLElement>(":scope > .ghg-content");
      if (!heading || !button || !content) {
        continue;
      }
      const around = element.parentElement?.closest(".ghg-block");
      const parent = around ? (byElement.get(around) ?? null) : null;
      const block = new DomBlock(heading.id, button, content, parent);
      byElement.set(element, block);
      this.found.push(block);
    }
    // A folder page's features, each known by its link's address.
    for (const element of document.querySelectorAll(".ghg-feature")) {
      const link = element.querySelector<HTMLAnchorElement>(":scope > a[href]");
      const button = element.querySelector<HTMLButtonElement>(":scope > button.ghg-toggle");
      const content = element.querySelector<HTMLElement>(":scope > .ghg-content");
      if (!link || !button || !content) {
        continue;
      }
      this.found.push(new DomBlock(link.getAttribute("href") ?? "", button, content, null));
    }
  }

  blocks(): readonly Block[] {
    return this.found;
  }

  target(): string | null {
    return anchorOf(this.document.location.hash);
  }

  onToggle(handler: (block: Block) => void): void {
    for (const block of this.found) {
      block.button.addEventListener("click", () => handler(block));
    }
  }

  onCollapseAll(handler: () => void): void {
    this.onClick(".ghg-collapse-all", handler);
  }

  onExpandAll(handler: () => void): void {
    this.onClick(".ghg-expand-all", handler);
  }

  onFollow(handler: (anchor: string) => void): void {
    const view = this.document.defaultView;
    view?.addEventListener("hashchange", () => {
      const anchor = anchorOf(this.document.location.hash);
      if (anchor !== null) {
        handler(anchor);
      }
    });
    // A link to the anchor already in the address changes nothing, so it is caught as it is followed.
    this.document.addEventListener("click", (event) => {
      // A click that opens another tab or window, or that a script stopped, changes nothing here.
      if (event.defaultPrevented || event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) {
        return;
      }
      const target = event.target as Element | null;
      const link = typeof target?.closest === "function" ? target.closest("a[href^='#']") : null;
      const anchor = link ? anchorOf(link.getAttribute("href") ?? "") : null;
      if (anchor !== null) {
        handler(anchor);
      }
    });
  }

  private onClick(selector: string, handler: () => void): void {
    this.document.querySelector(selector)?.addEventListener("click", handler);
  }
}

/** A link's anchor, which the address may hold percent-encoded. */
function anchorOf(hash: string): string | null {
  if (!hash.startsWith("#") || hash.length === 1) {
    return null;
  }
  try {
    return decodeURIComponent(hash.slice(1));
  } catch {
    return hash.slice(1);
  }
}
