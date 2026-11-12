// Adapter: the search box port over the search form src/policies/rendering/search_box.html writes.

import type { Result, SearchBox, SearchWords } from "../ports/search_box.ts";

/** Long enough for screen readers to hear the status emptied, so that the same words are read again. */
const REANNOUNCE_MILLISECONDS = 50;

export class DomSearchBox implements SearchBox {
  private readonly form: HTMLFormElement;
  private readonly field: HTMLInputElement;
  private readonly status: HTMLElement;
  private readonly results: HTMLOListElement;
  private pendingStatus: number | null = null;

  /** Null when the page has no search form, or not the whole of one. */
  static find(document: Document): DomSearchBox | null {
    const form = document.querySelector<HTMLFormElement>("form.ghg-search");
    const field = form?.querySelector<HTMLInputElement>("input.ghg-search-field");
    const status = form?.querySelector<HTMLElement>(".ghg-search-status");
    const results = form?.querySelector<HTMLOListElement>("ol.ghg-search-results");
    if (form && field && status && results && form.dataset["root"] !== undefined && form.dataset["shown"] !== undefined) {
      return new DomSearchBox(form, field, status, results);
    }
    return null;
  }

  private constructor(form: HTMLFormElement, field: HTMLInputElement, status: HTMLElement, results: HTMLOListElement) {
    this.form = form;
    this.field = field;
    this.status = status;
    this.results = results;
    form.addEventListener("submit", (event) => event.preventDefault());
  }

  words(): SearchWords {
    const data = this.form.dataset;
    const text = (key: string): string => data[key] ?? "";
    return {
      shown: Number(data["shown"]),
      loading: text("loading"),
      failed: text("failed"),
      none: text("none"),
      count: text("count"),
      countLimited: text("countLimited"),
      noName: text("noName"),
      feature: text("feature"),
      rule: text("rule"),
      scenario: text("scenario"),
      scenarioInRule: text("scenarioInRule"),
      scenarioOutline: text("scenarioOutline"),
      scenarioOutlineInRule: text("scenarioOutlineInRule"),
    };
  }

  onInput(handler: (text: string) => void): void {
    this.field.addEventListener("input", () => handler(this.field.value));
  }

  onEnter(handler: () => void): void {
    this.onKey("Enter", handler);
  }

  onEscape(handler: () => void): void {
    this.onKey("Escape", handler);
  }

  setStatus(text: string): void {
    const view = this.form.ownerDocument.defaultView;
    if (this.pendingStatus !== null) {
      view?.clearTimeout(this.pendingStatus);
      this.pendingStatus = null;
    }
    if (text === "" || this.status.textContent !== text || view === null) {
      this.status.textContent = text;
      return;
    }
    this.status.textContent = "";
    this.pendingStatus = view.setTimeout(() => {
      this.pendingStatus = null;
      this.status.textContent = text;
    }, REANNOUNCE_MILLISECONDS);
  }

  showResults(results: readonly Result[]): void {
    const document = this.form.ownerDocument;
    const root = this.form.dataset["root"] ?? "";
    const items = results.map((result) => {
      const link = document.createElement("a");
      link.setAttribute("href", root + result.address);
      for (const piece of result.label) {
        if (piece.isName) {
          const name = document.createElement("bdi");
          name.textContent = piece.text;
          link.append(name);
        } else {
          link.append(piece.text);
        }
      }
      const item = document.createElement("li");
      item.append(link);
      return item;
    });
    this.results.replaceChildren(...items);
    this.results.hidden = items.length === 0;
  }

  clear(): void {
    this.field.value = "";
  }

  focusFirstResult(): void {
    this.results.querySelector("a")?.focus();
  }

  private onKey(key: string, handler: () => void): void {
    this.field.addEventListener("keydown", (event) => {
      // Safari gives the Enter that ends a composition isComposing false, but keyCode 229.
      if (event.key === key && !event.isComposing && event.keyCode !== 229) {
        event.preventDefault();
        handler();
      }
    });
  }
}
