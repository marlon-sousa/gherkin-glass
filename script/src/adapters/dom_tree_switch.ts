// Adapter: the tree switch over the page's markup, as src/policies/rendering/page_frame.html and page_layout.html write it: the checkbox, the button and the landmark they show or hide.

import type { FocusPlace, TreeSwitch } from "../ports/tree.ts";

export class DomTreeSwitch implements TreeSwitch {
  private readonly checkbox: HTMLInputElement | null;
  private readonly button: HTMLButtonElement | null;
  private readonly landmark: HTMLElement | null;
  /**
   * The control the stylesheet hid with the focus on it, which the browser
   * then dropped to the body, often before the resize is reported.
   */
  private dropped: FocusPlace = null;

  constructor(document: Document) {
    this.checkbox = document.querySelector<HTMLInputElement>("input.ghg-show-tree");
    this.button = document.querySelector<HTMLButtonElement>("button.ghg-tree-button");
    this.landmark = document.querySelector<HTMLElement>("nav.ghg-tree");
    this.keepDropped(this.checkbox, "checkbox");
    this.keepDropped(this.button, "button");
    document.addEventListener("focusin", () => {
      this.dropped = null;
    });
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

  focus(): FocusPlace {
    const active = this.landmark?.ownerDocument.activeElement ?? null;
    if (active === null || active === active.ownerDocument.body) {
      return this.dropped;
    }
    if (this.landmark?.contains(active)) {
      return "tree";
    }
    if (active === this.checkbox) {
      return "checkbox";
    }
    return active === this.button ? "button" : null;
  }

  focusButton(): void {
    this.button?.focus();
  }

  focusCheckbox(): void {
    this.checkbox?.focus();
  }

  /** A blur to nowhere of a control no longer shown is one a resize hid; a click elsewhere leaves it shown. */
  private keepDropped(control: HTMLElement | null, place: FocusPlace): void {
    control?.addEventListener("blur", (event) => {
      if (event.relatedTarget === null && !control.checkVisibility()) {
        this.dropped = place;
      }
    });
  }
}
