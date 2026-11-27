// Adapter: the search page port over the page of search results dom_results_page.tsx draws, its words on the element that holds the results and its field the search box's.

import { drawn } from "./jsx/drawing.ts";
import type { Child, Markup } from "./jsx/jsx_runtime.ts";
import type { ListedResult, PageWords, Part, ResultGroup, Run, SearchPage } from "../ports/search_page.ts";

export class DomSearchPage implements SearchPage {
  /** Null when the page is not the page of search results, or not the whole of one; `searched` is the words of the shell's address. */
  static find(document: Document, searched: string): DomSearchPage | null {
    const heading = document.querySelector<HTMLElement>("h1.ghg-search-heading");
    const field = document.querySelector<HTMLInputElement>("form.ghg-search input.ghg-search-field");
    const status = document.querySelector<HTMLElement>(".ghg-results-status");
    const groups = document.querySelector<HTMLElement>(".ghg-search-groups");
    if (heading && field && status && groups) {
      return new DomSearchPage(document, heading, field, status, groups, searched);
    }
    return null;
  }

  private constructor(
    private readonly document: Document,
    private readonly heading: HTMLElement,
    private readonly field: HTMLInputElement,
    private readonly status: HTMLElement,
    private readonly groups: HTMLElement,
    private readonly searchedWords: string,
  ) {}

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
    return this.searchedWords;
  }

  showSearched(words: string, heading: readonly Part[], title: string): void {
    this.field.value = words;
    this.heading.replaceChildren(...drawn(this.document, partsOf(heading)));
    this.document.title = title;
  }

  setStatus(text: string): void {
    this.status.textContent = text;
  }

  showGroups(groups: readonly ResultGroup[]): void {
    const words = this.words();
    this.groups.replaceChildren(...drawn(this.document, groups.map((group) => groupOf(words, group))));
  }
}

function groupOf(words: PageWords, group: ResultGroup): Markup {
  return (
    <>
      <h2>{linkOf(group.address, group.heading)}</h2>
      <ul class="ghg-search-group">
        {group.results.map((result) => (
          <li>{resultOf(words, result)}</li>
        ))}
      </ul>
    </>
  );
}

function resultOf(words: PageWords, result: ListedResult): Markup {
  return (
    <>
      {linkOf(result.address, result.heading)}
      {result.within !== null && <p>{partsOf(result.within)}</p>}
      {result.steps.length > 0 && [
        <p>{words.matchingSteps}</p>,
        <ul>
          {result.steps.map((step) => (
            <li>{nameOf(step)}</li>
          ))}
        </ul>,
      ]}
      {namesOf(words, words.matchingTags, result.tags)}
      {namesOf(words, words.matchingCells, result.cells)}
    </>
  );
}

/** A paragraph of `texts` after `label`, or nothing when there are none. */
function namesOf(words: PageWords, label: string, texts: readonly (readonly Run[])[]): Markup | null {
  if (texts.length === 0) {
    return null;
  }
  return <p>{[`${label} `, texts.map((text, at) => [at > 0 && words.separator, nameOf(text)])]}</p>;
}

function linkOf(address: string, text: readonly Part[]): Markup {
  return <a href={address}>{partsOf(text)}</a>;
}

/** Each part's runs, a name's isolated in a `bdi` of its own. */
function partsOf(parts: readonly Part[]): Child {
  return parts.map((part) => (part.isName ? nameOf(part.runs) : runsOf(part.runs)));
}

function nameOf(runs: readonly Run[]): Markup {
  return <bdi>{runsOf(runs)}</bdi>;
}

/** Each run as text, a marked one in a `mark` of its own. */
function runsOf(runs: readonly Run[]): Child {
  return runs.map((run) => (run.marked ? <mark>{run.text}</mark> : run.text));
}
