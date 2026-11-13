// Adapter: the tree switch over the page's markup, as src/policies/rendering/page_frame.html and page_layout.html write it: the checkbox, the button and the landmark they show or hide.

import type { TreeSwitch } from "../ports/tree.ts";

export class DomTreeSwitch implements TreeSwitch {
  private readonly checkbox: HTMLInputElement | null;
  private readonly button: HTMLButtonElement | null;
  private readonly landmark: HTMLElement | null;

  constructor(document: Document) {
    this.checkbox = document.querySelector<HTMLInputElement>("input.ghg-show-tree");
    this.button = document.querySelector<HTMLButtonElement>("button.ghg-tree-button");
    this.landmark = document.querySelector<HTMLElement>("nav.ghg-tree");
  }

  configured(): boolean {
    return this.checkbox?.checked ?? false;
  }

  setChecked(checked: boolean): void {
    if (this.checkbox) {
      this.checkbox.checked = checked;
    }
  }

  setOpen(open: boolean): void {
    this.button?.setAttribute("aria-expanded", String(open));
  }

  setShown(shown: boolean): void {
    if (this.landmark) {
      this.landmark.hidden = !shown;
    }
  }

  disable(): void {
    if (this.checkbox) {
      this.checkbox.disabled = true;
    }
    if (this.button) {
      this.button.disabled = true;
    }
  }

  onChange(handler: (checked: boolean) => void): void {
    const checkbox = this.checkbox;
    checkbox?.addEventListener("change", () => handler(checkbox.checked));
  }

  onPress(handler: () => void): void {
    this.button?.addEventListener("click", () => handler());
  }

  onEscape(handler: () => void): void {
    this.landmark?.addEventListener("keydown", (event) => {
      if (event.key === "Escape") {
        handler();
      }
    });
  }

  focusButton(): void {
    this.button?.focus();
  }
}
