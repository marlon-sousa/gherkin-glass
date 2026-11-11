// Policy: how search matches, and the words and address of each result and of the status.

import type { Options, SearchOptions } from "minisearch";

import type { SearchDocument } from "../ports/documents.ts";
import type { Piece, Result, SearchWords } from "../ports/search_box.ts";

/** A document with the number MiniSearch knows it by, its place in the documents. */
export type Indexed = SearchDocument & { readonly id: number };

export const INDEX_OPTIONS: Options<Indexed> = {
  fields: ["name", "tags", "steps"],
  extractField: (document, field) => {
    const value = (document as unknown as Record<string, unknown>)[field];
    return Array.isArray(value) ? value.join(" ") : (value as string);
  },
};

/** Every word must match, as a word's start or one letter in five away, rounded to the nearest. */
export const SEARCH_OPTIONS: SearchOptions = {
  combineWith: "AND",
  prefix: true,
  fuzzy: 0.2,
  boost: { name: 3, tags: 2, steps: 1 },
};

export const PAUSE_MILLISECONDS = 300;

export function status(words: SearchWords, count: number): string {
  if (count === 0) {
    return words.none;
  }
  const template = count > words.shown ? words.countLimited : words.count;
  return template.replaceAll("{count}", String(count));
}

export function result(words: SearchWords, document: SearchDocument): Result {
  return { label: label(words, document), address: address(document) };
}

function label(words: SearchWords, document: SearchDocument): Piece[] {
  const inRule = document.rule !== null;
  const template = {
    feature: words.feature,
    rule: words.rule,
    scenario: inRule ? words.scenarioInRule : words.scenario,
    "scenario-outline": inRule ? words.scenarioOutlineInRule : words.scenarioOutline,
  }[document.kind];
  return template
    .split(/(\{(?:name|feature|rule)\})/)
    .filter((part) => part !== "")
    .map((part) => {
      switch (part) {
        case "{name}":
          return name(words, document.name);
        case "{feature}":
          return name(words, document.feature);
        case "{rule}":
          return name(words, document.rule);
        default:
          return { text: part, isName: false };
      }
    });
}

/** A name as written, or the words for none. */
function name(words: SearchWords, written: string | null): Piece {
  return written === null || written === "" ? { text: words.noName, isName: false } : { text: written, isName: true };
}

function address(document: SearchDocument): string {
  const page = document.page.split("/").map(encodeURIComponent).join("/");
  return document.anchor === "" ? page : `${page}#${encodeURIComponent(document.anchor)}`;
}
