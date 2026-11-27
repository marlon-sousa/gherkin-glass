// Policy: how search indexes the documents and matches the words searched for, which the search box and the page of search results share.

import MiniSearch, { type Options, type SearchOptions, type SearchResult } from "minisearch";

import { pageByName, targetAddress } from "./address.ts";
import type { SearchDocument } from "../ports/documents.ts";
import type { Run } from "../ports/search_page.ts";

/** A document with the number MiniSearch knows it by, its place in the documents. */
export type Indexed = SearchDocument & { readonly id: number };

/** The fields searched, each a member of a document. */
export type Field = "name" | "tags" | "steps" | "cells";

const INDEX_OPTIONS: Options<Indexed> = {
  fields: ["name", "tags", "steps", "cells"],
  extractField: (document, field) => {
    const value = (document as unknown as Record<string, unknown>)[field];
    return Array.isArray(value) ? value.join(" ") : (value as string);
  },
};

/** Every word must match, as a word's start or one letter in five away, rounded to the nearest. */
const SEARCH_OPTIONS: SearchOptions = {
  combineWith: "AND",
  prefix: true,
  fuzzy: 0.2,
  boost: { name: 3, tags: 2, steps: 1, cells: 1 },
};

/** Where MiniSearch splits a text into words, as its default tokenizer does. */
const BETWEEN_WORDS = /([\n\r\p{Z}\p{P}]+)/u;

/** A document found, and the words of its text the search matched in each field, lower-cased. */
export interface Found {
  readonly document: SearchDocument;
  readonly matched: ReadonlyMap<Field, ReadonlySet<string>>;
}

export interface Index {
  /** Every document that holds every word of `text`, best match first. */
  search(text: string): readonly Found[];
}

export function indexOf(documents: readonly SearchDocument[]): Index {
  const search = new MiniSearch<Indexed>(INDEX_OPTIONS);
  search.addAll(documents.map((document, id) => ({ ...document, id })));
  return {
    search: (text) => search.search(text, SEARCH_OPTIONS).flatMap((match) => found(documents, match)),
  };
}

function found(documents: readonly SearchDocument[], match: SearchResult): Found[] {
  const document = documents[match.id as number];
  if (document === undefined) {
    return [];
  }
  const matched = new Map<Field, Set<string>>();
  for (const [term, fields] of Object.entries(match.match)) {
    for (const field of fields as Field[]) {
      matched.set(field, (matched.get(field) ?? new Set<string>()).add(term));
    }
  }
  return [{ document, matched }];
}

/** `text` in runs, each word among `matched`, compared lower-cased, a marked run of its own. */
export function marked(text: string, matched: ReadonlySet<string> | undefined): Run[] {
  const runs: Run[] = [];
  for (const part of text.split(BETWEEN_WORDS)) {
    if (part === "") {
      continue;
    }
    const isMatch = matched?.has(part.toLowerCase()) ?? false;
    const last = runs.at(-1);
    if (last !== undefined && !last.marked && !isMatch) {
      runs[runs.length - 1] = { text: last.text + part, marked: false };
    } else {
      runs.push({ text: part, marked: isMatch });
    }
  }
  return runs;
}

/** Whether `runs` hold a word the search matched. */
export function holdsMatch(runs: readonly Run[]): boolean {
  return runs.some((run) => run.marked);
}

/** How the addresses of search's links are made, those of pages ghg wrote whole or those of a shell's. */
export interface SearchAddresses {
  /** The address of `document`'s entry, percent-encoded. */
  entry(document: SearchDocument): string;
  /** The page of search results for `words`, its words encoded as a form encodes them. */
  results(words: string): string;
}

/** Relative to the site's root, each document's page a file of its own, named with `.html`. */
export const WRITTEN_ADDRESSES: SearchAddresses = {
  entry: (document) => {
    const page = document.page.split("/").map(encodeURIComponent).join("/");
    return document.anchor === "" ? page : `${page}#${encodeURIComponent(document.anchor)}`;
  },
  results: (words) => `search.html?${new URLSearchParams({ q: words }).toString()}`,
};

/** A shell's own addresses, from its `#`, as rule 235 makes them. */
export const SHELL_ADDRESSES: SearchAddresses = {
  entry: (document) => targetAddress(pageByName(document.page), document.anchor === "" ? null : document.anchor),
  results: (words) => targetAddress({ kind: "search", words }),
};
