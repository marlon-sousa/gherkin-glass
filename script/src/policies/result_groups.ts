// Policy: what the page of search results shows: its heading and title for the words searched for, its status, and the results grouped by feature, each with the parts of it the search matched.

import type { SearchDocument } from "../ports/documents.ts";
import type { KindHeadings, ListedResult, PageWords, Part, ResultGroup, Run } from "../ports/search_page.ts";
import { type Field, type Found, type SearchAddresses, WRITTEN_ADDRESSES, holdsMatch, marked } from "./search_index.ts";

/** The page's heading and title for `text`, the words of its address without the spaces at either end. */
export function searchedHeading(words: PageWords, text: string): { heading: Part[]; title: string } {
  return {
    heading: filled(words.heading, "{words}", [plainName(text)]),
    title: words.title.replaceAll("{words}", () => text),
  };
}

export function pageStatus(words: PageWords, groups: readonly ResultGroup[]): string {
  const count = groups.reduce((sum, group) => sum + group.results.length, 0);
  if (count === 0) {
    return words.none;
  }
  return words.count.replaceAll("{count}", String(count)).replaceAll("{features}", String(groups.length));
}

/** `found`, best match first, grouped by feature, the groups in the order of their best result. */
export function groupedResults(words: PageWords, found: readonly Found[], addresses: SearchAddresses = WRITTEN_ADDRESSES): ResultGroup[] {
  const groups = new Map<string, { heading: Part[]; address: string; results: ListedResult[] }>();
  for (const one of found) {
    const page = one.document.page;
    const group = groups.get(page) ?? { heading: featureHeading(words.headings, one.document), address: addresses.entry({ ...one.document, anchor: "" }), results: [] };
    group.results.push(listed(words, one, addresses));
    groups.set(page, group);
  }
  return [...groups.values()];
}

function listed(words: PageWords, { document, matched }: Found, addresses: SearchAddresses): ListedResult {
  const marks = (field: Field) => (text: string) => marked(text, matched.get(field));
  return {
    heading: blockHeading(words.headings, document, marks("name")(document.name)),
    address: addresses.entry(document),
    within: within(words, document),
    steps: document.steps.map(marks("steps")).filter(holdsMatch),
    tags: document.tags.map(marks("tags")).filter(holdsMatch),
    cells: document.cells.map(marks("cells")).filter(holdsMatch),
  };
}

/** The paragraph naming the rule and the outline `document` is in, or null when it is in neither. */
function within(words: PageWords, document: SearchDocument): Part[] | null {
  const blocks = [
    document.rule === null ? null : named(words.headings.rule, words.headings.unnamedRule, document.rule),
    document.outline === null ? null : named(words.headings.scenarioOutline, words.headings.unnamedScenarioOutline, document.outline),
  ].filter((block) => block !== null);
  if (blocks.length === 0) {
    return null;
  }
  const separator: Part = { runs: [{ text: words.separator, marked: false }], isName: false };
  const joined = blocks.flatMap((block, at) => (at === 0 ? block : [separator, ...block]));
  return filled(words.within, "{blocks}", joined);
}

function featureHeading(headings: KindHeadings, document: SearchDocument): Part[] {
  return named(headings.feature, headings.unnamedFeature, document.feature ?? document.name);
}

/** The heading of `document`'s block, its name in `name` as the feature page's heading words it. */
function blockHeading(headings: KindHeadings, document: SearchDocument, name: readonly Run[]): Part[] {
  const [template, unnamed] = {
    feature: [headings.feature, headings.unnamedFeature],
    rule: [headings.rule, headings.unnamedRule],
    background: [headings.background, headings.unnamedBackground],
    scenario: [headings.scenario, headings.unnamedScenario],
    "scenario-outline": [headings.scenarioOutline, headings.unnamedScenarioOutline],
    examples: [headings.examples, headings.numberedExamples.replaceAll("{place}", String(document.place ?? 1))],
  }[document.kind] as [string, string];
  if (document.name.trim() === "") {
    return [plainWords(unnamed)];
  }
  return filled(template, "{name}", [{ runs: name, isName: true }]);
}

/** The heading `template` gives a block named `name`, or `unnamed` when it has none. */
function named(template: string, unnamed: string, name: string): Part[] {
  return name.trim() === "" ? [plainWords(unnamed)] : filled(template, "{name}", [plainName(name)]);
}

/** `template` with `parts` in the place of `placeholder`, the rest as gherkin-glass's own words. */
function filled(template: string, placeholder: string, parts: readonly Part[]): Part[] {
  return template
    .split(placeholder)
    .flatMap((words, at) => (at === 0 ? [words] : [parts, words]))
    .flatMap((piece) => (typeof piece === "string" ? (piece === "" ? [] : [plainWords(piece)]) : piece));
}

function plainWords(text: string): Part {
  return { runs: [{ text, marked: false }], isName: false };
}

function plainName(text: string): Part {
  return { runs: [{ text, marked: false }], isName: true };
}
