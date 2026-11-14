// Adapter: the page port over the page's markup, as feature_block.html, examples_item.html, feature_page.html and folder_feature.html in src/policies/rendering write it.

import type { Block, Page } from "../ports/page.ts";

class DomBlock implements Block {
  readonly anchor: string;
  readonly parent: Block | null;
  readonly expandedWhenUnchecked: boolean | null;
  readonly inScenario: boolean;
  readonly button: HTMLButtonElement;
  private readonly content: HTMLElement;

  constructor(anchor: string, button: HTMLButtonElement, content: HTMLElement, parent: Block | null, element: Element | null) {
    this.anchor = anchor;
    this.button = button;
    this.content = content;
    this.parent = parent;
    this.expandedWhenUnchecked = element === null ? null : expandedWhenUnchecked(element);
    this.inScenario = element?.classList.contains("ghg-background-in-scenario") ?? false;
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
      // An item of an outline's list of examples holds its button itself, with no heading.
      const heading = element.querySelector<HTMLElement>(":scope > :is(h2, h3, h4, h5)") ?? element;
      const button = heading.querySelector<HTMLButtonElement>(":scope > button.ghg-toggle");
      const content = element.querySelector<HTMLElement>(":scope > .ghg-content");
      if (!button || !content) {
        continue;
      }
      const around = element.parentElement?.closest(".ghg-block");
      const parent = around ? (byElement.get(around) ?? null) : null;
      const block = new DomBlock(heading.id, button, content, parent, element);
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
      this.found.push(new DomBlock(link.getAttribute("href") ?? "", button, content, null, null));
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
      const link = typeof target?.closest === "function" ? target.closest<HTMLAnchorElement>("a[href]") : null;
      const anchor = link ? this.anchorOnPage(link) : null;
      if (anchor !== null) {
        handler(anchor);
      }
    });
  }

  /** The anchor a link leads to on this very page, such as the folder tree's, whatever its address is relative to. */
  private anchorOnPage(link: HTMLAnchorElement): string | null {
    let address: URL;
    try {
      address = new URL(link.getAttribute("href") ?? "", this.document.location.href);
    } catch {
      return null;
    }
    const here = this.document.location;
    return address.origin === here.origin && address.pathname === here.pathname ? anchorOf(address.hash) : null;
  }

  private onClick(selector: string, handler: () => void): void {
    this.document.querySelector(selector)?.addEventListener("click", handler);
  }
}

function expandedWhenUnchecked(element: Element): boolean | null {
  switch (element.getAttribute("data-expanded-when-unchecked")) {
    case "true":
      return true;
    case "false":
      return false;
    default:
      return null;
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
