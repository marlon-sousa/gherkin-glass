// Adapter: the words port over the catalogue's messages in Fluent's syntax, formatted by @fluent/bundle as Rust's i18n-embed formats them, without isolation marks.

import { FluentBundle, FluentResource } from "@fluent/bundle";

import type { Argument, Parts, WordPart, Words } from "../ports/words.ts";

/** Stand for feature text in a message's words, to find where it goes; no message of the catalogue holds them. */
const MARK_BASE = 0xe000;

export class FluentWords implements Words {
  private readonly bundle: FluentBundle;

  /** `source` is the catalogue's messages, as a `.ftl` file holds them, in English, the only language for now. */
  constructor(source: string) {
    this.bundle = new FluentBundle("en", { useIsolating: false });
    this.bundle.addResource(new FluentResource(source));
  }

  text(id: string, args: Readonly<Record<string, Argument>> = {}): string {
    const message = this.bundle.getMessage(id);
    if (message?.value == null) {
      return id;
    }
    return this.bundle.formatPattern(message.value, { ...args }, []);
  }

  parts(id: string, args: Readonly<Record<string, Argument>>, texts: Readonly<Record<string, Parts>>): Parts {
    const named = Object.entries(texts);
    const marks = named.map((_, place) => String.fromCodePoint(MARK_BASE + place));
    const marked = Object.fromEntries(named.map(([key], place) => [key, marks[place] ?? ""]));
    const words = this.text(id, { ...args, ...marked });
    const parts: WordPart[] = [];
    let rest = words;
    for (;;) {
      const next = firstMark(rest, marks);
      if (next === null) {
        break;
      }
      parts.push({ text: rest.slice(0, next.at), isolated: false }, ...(named[next.place]?.[1] ?? []));
      rest = rest.slice(next.at + 1);
    }
    parts.push({ text: rest, isolated: false });
    return parts.filter((part) => part.text !== "");
  }
}

function firstMark(words: string, marks: readonly string[]): { at: number; place: number } | null {
  const found = marks.map((mark, place) => ({ at: words.indexOf(mark), place })).filter((mark) => mark.at >= 0);
  return found.reduce<{ at: number; place: number } | null>((first, mark) => (first === null || mark.at < first.at ? mark : first), null);
}
