// Adapter: the search page port over the page of search results src/policies/rendering/search_page.html writes, its words on the element that holds the results and its field the search box's.

import type { ListedResult, PageWords, Part, ResultGroup, Run, SearchPage } from "../ports/search_page.ts";

export class DomSearchPage implements SearchPage {
  private readonly document: Document;
  private readonly heading: HTMLElement;
  private readonly field: HTMLInputElement;
  private readonly status: HTMLElement;
  private readonly groups: HTMLElement;

  /** Null when the page is not the page of search results, or not the whole of one. */
  static find(document: Document): DomSearchPage | null {
    const heading = document.querySelector<HTMLElement>("h1.ghg-search-heading");
    const field = document.querySelector<HTMLInputElement>("form.ghg-search input.ghg-search-field");
    const status = document.querySelector<HTMLElement>(".ghg-results-status");
    const groups = document.querySelector<HTMLElement>(".ghg-search-groups");
    if (heading && field && status && groups) {
      return new DomSearchPage(document, heading, field, status, groups);
    }
    return null;
  }

  private constructor(document: Document, heading: HTMLElement, field: HTMLInputElement, status: HTMLElement, groups: HTMLElement) {
    this.document = document;
    this.heading = heading;
    this.field = field;
    this.status = status;
    this.groups = groups;
  }

  words(): PageWords {
    const data = this.groups.dataset;
    const text = (key: string): string => data[key] ?? "";
    return {
      heading: text("heading"),
      title: text("title"),
      loading: text("loading"),
      failed: text("failed"),
      none: text("none"),
      count: text("count"),
      within: text("within"),
      separator: text("separator"),
      matchingSteps: text("matchingSteps"),
      matchingTags: text("matchingTags"),
      matchingCells: text("matchingCells"),
      headings: {
        feature: text("feature"),
        unnamedFeature: text("unnamedFeature"),
        rule: text("rule"),
        unnamedRule: text("unnamedRule"),
        background: text("background"),
        unnamedBackground: text("unnamedBackground"),
        scenario: text("scenario"),
        unnamedScenario: text("unnamedScenario"),
        scenarioOutline: text("scenarioOutline"),
        unnamedScenarioOutline: text("unnamedScenarioOutline"),
        examples: text("examples"),
        numberedExamples: text("numberedExamples"),
      },
    };
  }

  searched(): string {
    const address = this.document.defaultView?.location.href ?? "";
    return new URL(address, "https://example.invalid/").searchParams.get("q") ?? "";
  }

  showSearched(words: string, heading: readonly Part[], title: string): void {
    this.field.value = words;
    this.heading.replaceChildren(...this.parts(heading));
    this.document.title = title;
  }

  setStatus(text: string): void {
    this.status.textContent = text;
  }

  showGroups(groups: readonly ResultGroup[]): void {
    const words = this.words();
    this.groups.replaceChildren(...groups.flatMap((group) => this.group(words, group)));
  }

  private group(words: PageWords, group: ResultGroup): HTMLElement[] {
    const heading = this.element("h2", [this.link(group.address, group.heading)]);
    const list = this.element(
      "ul",
      group.results.map((result) => this.element("li", this.result(words, result))),
    );
    list.className = "ghg-search-group";
    return [heading, list];
  }

  private result(words: PageWords, result: ListedResult): HTMLElement[] {
    const parts: HTMLElement[] = [this.link(result.address, result.heading)];
    if (result.within !== null) {
      parts.push(this.element("p", this.parts(result.within)));
    }
    if (result.steps.length > 0) {
      parts.push(
        this.element("p", [words.matchingSteps]),
        this.element(
          "ul",
          result.steps.map((step) => this.element("li", [this.name(step)])),
        ),
      );
    }
    for (const [label, texts] of [
      [words.matchingTags, result.tags],
      [words.matchingCells, result.cells],
    ] as const) {
      if (texts.length > 0) {
        const named = texts.flatMap((text, at) => (at === 0 ? [this.name(text)] : [words.separator, this.name(text)]));
        parts.push(this.element("p", [`${label} `, ...named]));
      }
    }
    return parts;
  }

  private link(address: string, text: readonly Part[]): HTMLAnchorElement {
    const link = this.element("a", this.parts(text));
    link.setAttribute("href", address);
    return link;
  }

  /** Each part's runs, a name's isolated in a `bdi` of its own. */
  private parts(parts: readonly Part[]): (Node | string)[] {
    return parts.flatMap((part) => (part.isName ? [this.name(part.runs)] : this.runs(part.runs)));
  }

  private name(runs: readonly Run[]): HTMLElement {
    return this.element("bdi", this.runs(runs));
  }

  /** Each run as text, a marked one in a `mark` of its own. */
  private runs(runs: readonly Run[]): (Node | string)[] {
    return runs.map((run) => (run.marked ? this.element("mark", [run.text]) : run.text));
  }

  private element<K extends keyof HTMLElementTagNameMap>(name: K, children: readonly (Node | string)[]): HTMLElementTagNameMap[K] {
    const element = this.document.createElement(name);
    element.append(...children);
    return element;
  }
}
