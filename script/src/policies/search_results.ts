// Policy: the words and address of each result of the search box, of its status and of its link to the page of search results, and which result the arrow keys choose.

import type { SearchDocument } from "../ports/documents.ts";
import type { Piece, Result, SearchWords } from "../ports/search_box.ts";
import { type SearchAddresses, WRITTEN_ADDRESSES } from "./search_index.ts";

export const PAUSE_MILLISECONDS = 300;

export function status(words: SearchWords, count: number): string {
  if (count === 0) {
    return words.none;
  }
  const template = count > words.shown ? words.countLimited : words.count;
  return template.replaceAll("{count}", String(count));
}

export function result(words: SearchWords, document: SearchDocument, addresses: SearchAddresses = WRITTEN_ADDRESSES): Result {
  return { label: label(words, document), address: addresses.entry(document) };
}

/** The link to the page of search results for `text`, or null when it holds only spaces. */
export function allResults(words: SearchWords, text: string, addresses: SearchAddresses = WRITTEN_ADDRESSES): Result | null {
  const searched = text.trim();
  if (searched === "") {
    return null;
  }
  return { label: filled(words.allResults, { "{words}": { text: searched, isName: true } }), address: addresses.results(searched) };
}

/**
 * The result chosen after moving `step`, 1 down or -1 up, from `chosen` among `count` results:
 * from none, the first or the last, and past either end, none.
 */
export function moved(chosen: number | null, count: number, step: 1 | -1): number | null {
  if (count === 0) {
    return null;
  }
  if (chosen === null) {
    return step === 1 ? 0 : count - 1;
  }
  const next = chosen + step;
  return next < 0 || next >= count ? null : next;
}

function label(words: SearchWords, document: SearchDocument): Piece[] {
  const inRule = document.rule !== null;
  const template = {
    feature: words.feature,
    rule: words.rule,
    background: inRule ? words.backgroundInRule : words.background,
    scenario: inRule ? words.scenarioInRule : words.scenario,
    "scenario-outline": inRule ? words.scenarioOutlineInRule : words.scenarioOutline,
    examples: inRule ? words.examplesInRule : words.examples,
  }[document.kind];
  return filled(template, {
    "{name}": blockName(words, document),
    "{feature}": name(words, document.feature),
    "{rule}": name(words, document.rule),
    "{outline}": name(words, document.outline),
  });
}

/** `template` with each placeholder of `pieces` in its place, the rest as words. */
function filled(template: string, pieces: Readonly<Record<string, Piece>>): Piece[] {
  return template
    .split(/(\{[a-z]+\})/)
    .filter((part) => part !== "")
    .map((part) => pieces[part] ?? { text: part, isName: false });
}

/** The block's own name, where an examples block without one is numbered by its place. */
function blockName(words: SearchWords, document: SearchDocument): Piece {
  if (document.kind === "examples" && document.name.trim() === "") {
    return { text: words.numberedExamples.replaceAll("{place}", String(document.place ?? 1)), isName: false };
  }
  return name(words, document.name);
}

/** A name as written, or the words for none. */
function name(words: SearchWords, written: string | null): Piece {
  return written === null || written.trim() === "" ? { text: words.noName, isName: false } : { text: written, isName: true };
}
