// Policy: what the script reads from each file of the site's data, checked member by member, in the store's format.

import type { Block, Descriptor, DocString, FeatureEntry, PageEntry, PageIndex, PageObject, Part, Run, Sentences, StartStates, Step, StoreObject, TagEntry, TagLink, VersionList } from "../ports/store.ts";

/** The format of the store this script reads. */
export const STORE_FORMAT = 1;

/** Returns what `data` holds, or throws a `TypeError` when it is not that. */
type Reader<T> = (data: unknown) => T;

type Members<T> = { readonly [Key in keyof T]: Reader<T[Key]> };

function mismatch(what: string): never {
  throw new TypeError(`the site's data holds no ${what} where one is read`);
}

const text: Reader<string> = (data) => (typeof data === "string" ? data : mismatch("text"));
const truth: Reader<boolean> = (data) => (typeof data === "boolean" ? data : mismatch("true or false"));
const whole: Reader<number> = (data) => (typeof data === "number" && Number.isSafeInteger(data) && data >= 0 ? data : mismatch("whole number"));
const format: Reader<number> = (data) => (whole(data) === STORE_FORMAT ? STORE_FORMAT : mismatch(`format ${STORE_FORMAT}`));

function list<T>(item: Reader<T>): Reader<readonly T[]> {
  return (data) => (Array.isArray(data) ? data.map(item) : mismatch("list"));
}

function nullable<T>(value: Reader<T>): Reader<T | null> {
  return (data) => (data === null ? null : value(data));
}

function oneOf<T extends string>(...values: readonly T[]): Reader<T> {
  return (data) => values.find((value) => value === data) ?? mismatch(values.join(" or "));
}

/** Members the script does not read are passed over, as a later ghg of the same format may add them. */
function object<T>(members: Members<T>): Reader<T> {
  return (data) => {
    if (typeof data !== "object" || data === null || Array.isArray(data)) {
      return mismatch("object");
    }
    const found = new Map(Object.entries(data));
    const read = Object.entries<Reader<unknown>>(members).map(([key, member]) => [key, member(found.get(key))]);
    return Object.fromEntries(read) as T;
  };
}

/** A reader of what refers to itself, made when it is first used. */
function later<T>(reader: () => Reader<T>): Reader<T> {
  return (data) => reader()(data);
}

const table = nullable(list(list(text)));

const part: Reader<Part> = object<Part>({
  kind: oneOf("text", "parameter", "number", "placeholder"),
  text,
  parts: later(() => list(part)),
});

const docString = object<DocString>({ media_type: nullable(text), content: text });

const step = object<Step>({
  keyword: text,
  text,
  kind: oneOf("given", "when", "then"),
  sentence: text,
  slots: list(text),
  parts: list(part),
  table,
  doc_string: nullable(docString),
});

const run = object<Run>({ anchor: text, steps: list(step) });

const block: Reader<Block> = object<Block>({
  kind: oneOf("feature", "background", "rule", "scenario", "scenario-outline", "examples"),
  keyword: text,
  note: truth,
  name: text,
  description: text,
  tags: list(text),
  anchor: text,
  hash: text,
  blocks: later(() => list(block)),
  steps: list(step),
  examples: later(() => list(block)),
  place: nullable(whole),
  table,
  runs: list(run),
});

export const readObject: Reader<StoreObject> = object<StoreObject>({ format, language: text, lang: text, feature: block });

const featureEntry = object<FeatureEntry>({ path: text, page: text, object: text, lines: list(whole), source: nullable(text) });
const tagLink = object<TagLink>({ address: text, text });
const tagEntry = object<TagEntry>({ tag: text, page: text, link: nullable(tagLink) });
const startStates = object<StartStates>({ collapse_rules: truth, collapse_scenarios: truth, show_tree: truth, inline_backgrounds: truth });
const sentences = object<Sentences>({ given: list(text), when: list(text), then: list(text) });

export const readDescriptor: Reader<Descriptor> = object<Descriptor>({
  format,
  ghg: text,
  version: nullable(text),
  preview: nullable(text),
  title: text,
  only: truth,
  start: startStates,
  features: list(featureEntry),
  tags: list(tagEntry),
  sentences,
});

export const readVersions: Reader<VersionList> = object<VersionList>({
  format,
  development: list(text),
  releases: list(text),
  pre_releases: list(text),
  precedence: list(list(text)),
});

const pageObject = object<PageObject>({ object: text, versions: list(text) });
const pageEntry = object<PageEntry>({ page: text, objects: list(pageObject) });

export const readPages: Reader<PageIndex> = object<PageIndex>({ format, pages: list(pageEntry) });
