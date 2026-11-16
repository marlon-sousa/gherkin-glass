// Adapter: the search box port over the search form src/policies/rendering/search_box.html writes, its field a combobox over the list box of results.

import type { Piece, Result, SearchBox, SearchWords } from "../ports/search_box.ts";

/** Long enough for screen readers to hear the status emptied, so that the same words are read again. */
const REANNOUNCE_MILLISECONDS = 50;

/** The start of each option's `id`; the `_` keeps it apart from every anchor. */
const OPTION_ID = "ghg_search_result_";

export class DomSearchBox implements SearchBox {
  private readonly form: HTMLFormElement;
  private readonly field: HTMLInputElement;
  private readonly link: HTMLAnchorElement;
  private readonly status: HTMLElement;
  private readonly results: HTMLOListElement;
  private pendingStatus: number | null = null;

  /** Null when the page has no search form, or not the whole of one. */
  static find(document: Document): DomSearchBox | null {
    const form = document.querySelector<HTMLFormElement>("form.ghg-search");
    const field = form?.querySelector<HTMLInputElement>("input.ghg-search-field");
    const link = form?.querySelector<HTMLAnchorElement>("a.ghg-search-all");
    const status = form?.querySelector<HTMLElement>(".ghg-search-status");
    const results = form?.querySelector<HTMLOListElement>("ol.ghg-search-results");
    if (form && field && link && status && results && form.dataset["root"] !== undefined && form.dataset["shown"] !== undefined) {
      return new DomSearchBox(form, field, link, status, results);
    }
    return null;
  }

  private constructor(form: HTMLFormElement, field: HTMLInputElement, link: HTMLAnchorElement, status: HTMLElement, results: HTMLOListElement) {
    this.form = form;
    this.field = field;
    this.link = link;
    this.status = status;
    this.results = results;
    form.addEventListener("submit", (event) => event.preventDefault());
    // Choosing an option with the pointer keeps the focus in the field.
    results.addEventListener("mousedown", (event) => event.preventDefault());
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
      background: text("background"),
      backgroundInRule: text("backgroundInRule"),
      examples: text("examples"),
      examplesInRule: text("examplesInRule"),
      numberedExamples: text("numberedExamples"),
      allResults: text("allResults"),
    };
  }

  onInput(handler: (text: string) => void): void {
    this.field.addEventListener("input", () => handler(this.field.value));
  }

  onDown(handler: () => void): void {
    this.onKey("ArrowDown", handler);
  }

  onUp(handler: () => void): void {
    this.onKey("ArrowUp", handler);
  }

  onEnter(handler: () => void): void {
    this.onKey("Enter", handler);
  }

  onEscape(handler: () => void): void {
    this.onKey("Escape", handler);
  }

  onPick(handler: (place: number) => void): void {
    this.results.addEventListener("click", (event) => {
      const option = (event.target as Element | null)?.closest?.("[role=option]");
      const place = option ? [...this.results.children].indexOf(option) : -1;
      if (place >= 0) {
        handler(place);
      }
    });
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
    const options = results.map((result, place) => {
      const option = document.createElement("li");
      option.id = `${OPTION_ID}${place}`;
      option.setAttribute("role", "option");
      option.setAttribute("aria-selected", "false");
      option.append(...this.pieces(result.label));
      return option;
    });
    this.results.replaceChildren(...options);
    this.choose(null);
  }

  setOpen(open: boolean): void {
    this.results.hidden = !open;
    this.field.setAttribute("aria-expanded", String(open));
  }

  choose(place: number | null): void {
    for (const [at, option] of [...this.results.children].entries()) {
      option.setAttribute("aria-selected", String(at === place));
    }
    const option = place === null ? null : this.results.children.item(place);
    if (option === null) {
      this.field.removeAttribute("aria-activedescendant");
      return;
    }
    this.field.setAttribute("aria-activedescendant", option.id);
    option.scrollIntoView?.({ block: "nearest" });
  }

  showAllResults(link: Result | null): void {
    this.link.hidden = link === null;
    if (link === null) {
      this.link.replaceChildren();
      return;
    }
    this.link.replaceChildren(...this.pieces(link.label));
    this.link.setAttribute("href", this.root() + link.address);
  }

  fieldText(): string {
    return this.field.value;
  }

  clear(): void {
    this.field.value = "";
  }

  go(address: string): void {
    this.form.ownerDocument.defaultView?.location.assign(this.root() + address);
  }

  private root(): string {
    return this.form.dataset["root"] ?? "";
  }

  /** Each piece as text, a name isolated in a `bdi` of its own. */
  private pieces(label: readonly Piece[]): (Node | string)[] {
    return label.map((piece) => {
      if (!piece.isName) {
        return piece.text;
      }
      const name = this.form.ownerDocument.createElement("bdi");
      name.textContent = piece.text;
      return name;
    });
  }

  private onKey(key: string, handler: () => void): void {
    this.field.addEventListener("keydown", (event) => {
      // Safari gives the Enter that ends a composition isComposing false, but keyCode 229.
      if (event.key === key && !event.isComposing && event.keyCode !== 229 && !event.altKey && !event.ctrlKey && !event.metaKey) {
        event.preventDefault();
        handler();
      }
    });
  }
}
