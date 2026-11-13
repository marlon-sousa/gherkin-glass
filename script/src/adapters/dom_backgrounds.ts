// Adapter: the backgrounds port over the page's checkbox, as feature_page.html in src/policies/rendering writes it.

import type { Backgrounds } from "../ports/backgrounds.ts";

export class DomBackgrounds implements Backgrounds {
  private readonly checkbox: HTMLInputElement;
  private readonly document: Document;

  private constructor(document: Document, checkbox: HTMLInputElement) {
    this.document = document;
    this.checkbox = checkbox;
  }

  /** Null on a page without the checkbox, where no scenario starts with a background's steps. */
  static of(document: Document): DomBackgrounds | null {
    const checkbox = document.querySelector<HTMLInputElement>("input.ghg-inline-backgrounds");
    return checkbox ? new DomBackgrounds(document, checkbox) : null;
  }

  writtenInline(): boolean {
    // `checked` may hold what a browser restored from an earlier visit, not what the page says.
    return this.checkbox.defaultChecked;
  }

  setInline(inline: boolean): void {
    this.checkbox.checked = inline;
  }

  onSwitch(handler: (inline: boolean) => void): void {
    this.checkbox.addEventListener("change", () => handler(this.checkbox.checked));
  }

  onReturn(handler: () => void): void {
    this.document.defaultView?.addEventListener("pageshow", (event) => {
      if (event.persisted) {
        handler();
      }
    });
  }
}
