// Adapter: the page port over the page's markup, as templates/block.html and templates/feature.html write it.

import type { Block, Page } from "../ports/page.ts";

class DomBlock implements Block {
  readonly anchor: string;
  readonly parent: Block | null;
  readonly button: HTMLButtonElement;
  private readonly content: HTMLElement;

  constructor(heading: HTMLElement, button: HTMLButtonElement, content: HTMLElement, parent: Block | null) {
    this.anchor = heading.id;
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
      const heading = element.querySelector(":scope > :is(h2, h3, h4)");
      const button = heading?.querySelector(":scope > .ghg-toggle");
      const content = element.querySelector(":scope > .ghg-content");
      if (!(heading instanceof HTMLElement && button instanceof HTMLButtonElement && content instanceof HTMLElement)) {
        continue;
      }
      const around = element.parentElement?.closest(".ghg-block");
      const parent = around ? (byElement.get(around) ?? null) : null;
      const block = new DomBlock(heading, button, content, parent);
      byElement.set(element, block);
      this.found.push(block);
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
      const link = event.target instanceof Element ? event.target.closest("a[href^='#']") : null;
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
