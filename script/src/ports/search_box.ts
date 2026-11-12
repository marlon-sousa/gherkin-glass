// Port: the search box of the page, as src/policies/rendering/search_box.html writes it, and what the reader does in it.

/** The page's words for the search box, each `{count}`, `{name}`, `{feature}` and `{rule}` a placeholder. */
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
  onEnter(handler: () => void): void;
  onEscape(handler: () => void): void;
  setStatus(text: string): void;
  /** An empty list hides the results. */
  showResults(results: readonly Result[]): void;
  /** Empties the field. */
  clear(): void;
  /** Does nothing when there is no result. */
  focusFirstResult(): void;
}
