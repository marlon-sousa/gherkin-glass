// Builders of the site's data for the script's tests, each member defaulted as docs/BEHAVIOUR.md, rules 248 to 252, says of a block without it, so that a test names only what it is about.

import type { Block, BlockKind, Descriptor, DocString, FeatureEntry, Part, Run, Step, StepKind, StoreObject, Table, TagEntry } from "../../src/ports/store.ts";

export const text = (words: string): Part => ({ kind: "text", text: words, parts: [] });
export const number = (digits: string): Part => ({ kind: "number", text: digits, parts: [] });
export const placeholder = (name: string): Part => ({ kind: "placeholder", text: name, parts: [] });
export const parameter = (...parts: Part[]): Part => ({ kind: "parameter", text: parts.map((part) => part.text).join(""), parts });

export interface StepExtras {
  readonly table?: Table;
  readonly doc_string?: DocString;
  readonly sentence?: string;
  readonly slots?: readonly string[];
}

/** A step, its text the parts joined, its sentence its text unless given. */
export function step(keyword: string, kind: StepKind, parts: readonly Part[], extras: StepExtras = {}): Step {
  const words = parts.map((part) => part.text).join("");
  return {
    keyword,
    text: words,
    kind,
    sentence: extras.sentence ?? words,
    slots: extras.slots ?? [],
    parts,
    table: extras.table ?? null,
    doc_string: extras.doc_string ?? null,
  };
}

export function block(kind: BlockKind, keyword: string, name: string, anchor: string, extras: Partial<Block> = {}): Block {
  return {
    kind,
    keyword,
    note: false,
    name,
    description: "",
    tags: [],
    anchor,
    hash: `${anchor}-hash`,
    blocks: [],
    steps: [],
    examples: [],
    place: null,
    table: null,
    runs: [],
    ...extras,
  };
}

export function run(anchor: string, steps: readonly Step[]): Run {
  return { anchor, steps };
}

export function object(feature: Block, lang = "en", language = lang): StoreObject {
  return { format: 1, language, lang, feature };
}

export function entry(path: string, objectName: string, extras: Partial<FeatureEntry> = {}): FeatureEntry {
  return { path, page: path.replace(/\.[^./]+$/, ""), object: objectName, lines: [], source: null, ...extras };
}

export function tag(written: string, page: string, link: TagEntry["link"] = null): TagEntry {
  return { tag: written, page, link };
}

export function descriptor(title: string, features: readonly FeatureEntry[], extras: Partial<Descriptor> = {}): Descriptor {
  return {
    format: 1,
    ghg: "{version}",
    version: null,
    preview: null,
    title,
    only: false,
    start: { collapse_rules: false, collapse_scenarios: false, show_tree: true, inline_backgrounds: true },
    features,
    tags: [],
    sentences: { given: [], when: [], then: [] },
    ...extras,
  };
}
