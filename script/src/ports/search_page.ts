// Port: the page of search results, as src/adapters/dom_results_page.tsx draws it, which the script fills with what search finds.

/** A block's heading for each kind, with `{name}` for its name, and an unnamed examples block's with `{place}`. */
export interface KindHeadings {
  readonly feature: string;
  readonly unnamedFeature: string;
  readonly rule: string;
  readonly unnamedRule: string;
  readonly background: string;
  readonly unnamedBackground: string;
  readonly scenario: string;
  readonly unnamedScenario: string;
  readonly scenarioOutline: string;
  readonly unnamedScenarioOutline: string;
  readonly examples: string;
  readonly numberedExamples: string;
}

/** The page's words, each `{words}`, `{count}`, `{features}` and `{blocks}` a placeholder. */
export interface PageWords {
  readonly heading: string;
  readonly title: string;
  readonly loading: string;
  readonly failed: string;
  readonly none: string;
  readonly count: string;
  readonly within: string;
  readonly separator: string;
  readonly matchingSteps: string;
  readonly matchingTags: string;
  readonly matchingCells: string;
  readonly headings: KindHeadings;
}

/** A run of a text, which the search matched or not. */
export interface Run {
  readonly text: string;
  readonly marked: boolean;
}

/** A part of some words: gherkin-glass's own, or feature text, isolated from the words around it. */
export interface Part {
  readonly runs: readonly Run[];
  readonly isName: boolean;
}

export interface ListedResult {
  readonly heading: readonly Part[];
  /** Relative to the site's root, percent-encoded. */
  readonly address: string;
  /** The paragraph naming the blocks it is in, or null when it is in none. */
  readonly within: readonly Part[] | null;
  /** Each matched step, tag and cell, in its runs. */
  readonly steps: readonly (readonly Run[])[];
  readonly tags: readonly (readonly Run[])[];
  readonly cells: readonly (readonly Run[])[];
}

export interface ResultGroup {
  readonly heading: readonly Part[];
  /** The feature's page, relative to the site's root, percent-encoded. */
  readonly address: string;
  readonly results: readonly ListedResult[];
}

export interface SearchPage {
  words(): PageWords;
  /** The words of the page's address, as given, empty when it has none. */
  searched(): string;
  /** Writes the words searched for in the field, and the page's heading and title for them. */
  showSearched(words: string, heading: readonly Part[], title: string): void;
  setStatus(text: string): void;
  showGroups(groups: readonly ResultGroup[]): void;
}
