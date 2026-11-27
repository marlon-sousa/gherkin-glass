// Adapter: the page port over the page's markup, as dom_feature_page.tsx and dom_folder_page.tsx draw it.

import type { Block, Page } from "../ports/page.ts";

class DomBlock implements Block {
  readonly expandedWhenUnchecked: boolean | null;
  readonly inScenario: boolean;

  constructor(
    readonly anchor: string,
    readonly button: HTMLButtonElement,
    private readonly content: HTMLElement,
    readonly parent: Block | null,
    element: Element | null,
  ) {
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
  private readonly byElement = new Map<Element, DomBlock>();
  private readonly document: Document;

  /** `root` holds the blocks, the document's or a drawn page's `main`. */
  constructor(document: Document, root: ParentNode = document) {
    this.document = document;
    for (const element of root.querySelectorAll(".ghg-block")) {
      // An item of an outline's list of examples holds its button itself, with no heading.
      const heading = element.querySelector<HTMLElement>(":scope > :is(h2, h3, h4, h5)") ?? element;
      const button = heading.querySelector<HTMLButtonElement>(":scope > button.ghg-toggle");
      const content = element.querySelector<HTMLElement>(":scope > .ghg-content");
      if (!button || !content) {
        continue;
      }
      const around = element.parentElement?.closest(".ghg-block");
      const parent = around ? (this.byElement.get(around) ?? null) : null;
      const block = new DomBlock(heading.id, button, content, parent, element);
      this.byElement.set(element, block);
      this.found.push(block);
    }
    // A folder page's features, each known by its link's address.
    for (const element of root.querySelectorAll(".ghg-feature")) {
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

  holder(anchor: string): Block | null {
    const named = this.found.find((block) => block.anchor === anchor);
    if (named) {
      return named;
    }
    const around = this.document.getElementById(anchor)?.closest(".ghg-block");
    return around ? (this.byElement.get(around) ?? null) : null;
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
