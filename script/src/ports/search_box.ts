// Port: the search box of the page, a combobox over its list of results, as src/policies/rendering/search_box.html writes it, and what the reader does in it.

/** The page's words for the search box, each `{count}`, `{name}`, `{feature}`, `{rule}`, `{outline}`, `{place}` and `{words}` a placeholder. */
export interface SearchWords {
  readonly shown: number;
  readonly loading: string;
  readonly failed: string;
  readonly none: string;
  readonly count: string;
  readonly countLimited: string;
  readonly noName: string;
  readonly feature: string;
  readonly rule: string;
  readonly scenario: string;
  readonly scenarioInRule: string;
  readonly scenarioOutline: string;
  readonly scenarioOutlineInRule: string;
  readonly background: string;
  readonly backgroundInRule: string;
  readonly examples: string;
  readonly examplesInRule: string;
  readonly numberedExamples: string;
  readonly allResults: string;
}

/** A piece of a result's words; a name is isolated from the words around it. */
export interface Piece {
  readonly text: string;
  readonly isName: boolean;
}

export interface Result {
  readonly label: readonly Piece[];
  /** Relative to the site's root, percent-encoded. */
  readonly address: string;
}

export interface SearchBox {
  words(): SearchWords;
  onInput(handler: (text: string) => void): void;
  onDown(handler: () => void): void;
  onUp(handler: () => void): void;
  onEnter(handler: () => void): void;
  onEscape(handler: () => void): void;
  /** Called with the place of the result the reader chose with the pointer. */
  onPick(handler: (place: number) => void): void;
  setStatus(text: string): void;
  /** Replaces the options with `results`, none chosen. */
  showResults(results: readonly Result[]): void;
  /** Shows or hides the list, and says which in the field. */
  setOpen(open: boolean): void;
  /** Marks the option at `place` chosen, or none. */
  choose(place: number | null): void;
  /** Shows the link to the page of search results, or hides it for null. */
  showAllResults(link: Result | null): void;
  /** Empties the field. */
  clear(): void;
  /** Opens `address`, relative to the site's root. */
  go(address: string): void;
}
