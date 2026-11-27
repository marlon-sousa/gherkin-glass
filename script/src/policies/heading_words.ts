// Policy: the words of a block's heading and of the note of its keyword, in gherkin-glass's words around the feature text, from the catalogue.

import type { Block, BlockKind } from "../ports/store.ts";
import type { Argument, Parts, Words } from "../ports/words.ts";

/** The note's words before its button, the button's, with the keyword isolated, and those after it. */
export interface NoteWords {
  readonly before: string;
  readonly opener: Parts;
  readonly after: string;
}

export function isolated(text: string): Parts {
  return [{ text, isolated: true }];
}

export function plain(text: string): Parts {
  return text === "" ? [] : [{ text, isolated: false }];
}

/** A piece of a sentence of the catalogue: its own words, or the place of one of its texts, drawn in an element of its own. */
export type Piece = { readonly words: string } | { readonly slot: string };

/** The words of `id`, each of its texts named in `slots` a piece of its own. */
export function pieces(words: Words, id: string, args: Readonly<Record<string, Argument>>, slots: readonly string[]): Piece[] {
  const marks = slots.map((_, place) => String.fromCodePoint(0xe100 + place));
  const sentence = words.text(id, { ...args, ...Object.fromEntries(slots.map((slot, place) => [slot, marks[place] ?? ""])) });
  return [...sentence.split(/([\u{e100}-\u{e1ff}])/u)].flatMap((piece): Piece[] => {
    const place = marks.indexOf(piece);
    const slot = slots[place];
    return slot !== undefined ? [{ slot }] : piece === "" ? [] : [{ words: piece }];
  });
}

/** The words of `id` before and after its text `slot`, the other texts in `texts` put in, for a slot drawn in an element of its own. */
export function wordsAround(words: Words, id: string, args: Readonly<Record<string, Argument>>, texts: Readonly<Record<string, Parts>>, slot: string): { readonly before: Parts; readonly after: Parts } {
  const mark = "\u{e100}";
  const parts = words.parts(id, args, { ...texts, [slot]: plain(mark) });
  const at = parts.findIndex((part) => part.text === mark);
  return at < 0 ? { before: parts, after: [] } : { before: parts.slice(0, at), after: parts.slice(at + 1) };
}

/** The text of `parts`, as a title or an accessible name reads it. */
export function textOf(parts: Parts): string {
  return parts.map((part) => part.text).join("");
}

/** The heading of a block of `kind`: its kind, then its name, or its kind alone. */
export function heading(words: Words, kind: BlockKind, name: string): Parts {
  return named(words, kind, name, words.text("unnamed-heading", { kind }));
}

/** An examples block's heading, as `heading` makes it, an unnamed one numbered by its `place` among its outline's. */
export function examplesHeading(words: Words, name: string, place: number): Parts {
  return named(words, "examples", name, words.text("numbered-examples", { place }));
}

/** The heading of `block`, an examples block's numbered. */
export function blockHeading(words: Words, block: Block): Parts {
  return block.kind === "examples" ? examplesHeading(words, block.name, block.place ?? 1) : heading(words, block.kind, block.name);
}

function named(words: Words, kind: BlockKind, name: string, unnamed: string): Parts {
  return name === "" ? plain(unnamed) : words.parts("named-heading", { kind }, { name: isolated(name) });
}

/** The note of `block`'s keyword, when ghg found that its file writes another word than its kind's. */
export function keywordNote(words: Words, block: Block): NoteWords | null {
  if (!block.note) {
    return null;
  }
  const mark = "\u{e100}";
  const sentence = words.text("keyword-note", { kind: block.kind, opener: mark });
  const split = sentence.indexOf(mark);
  return {
    before: split < 0 ? sentence : sentence.slice(0, split),
    opener: words.parts("keyword-note-opener", {}, { keyword: isolated(block.keyword) }),
    after: split < 0 ? "" : sentence.slice(split + mark.length),
  };
}
