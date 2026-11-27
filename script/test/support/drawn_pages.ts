// What the tests of drawn pages share: the catalogue's words, a shell and a document to draw in, and the `main` the script is expected to draw.

import { readFileSync } from "node:fs";

import { Window } from "happy-dom";

import { FluentWords } from "../../src/adapters/fluent_words.ts";

/** The page messages of the catalogue, those after its `### ghg:page-messages` line, which a shell carries. */
export const CATALOGUE = readFileSync(new URL("../../../messages/en/gherkin-glass.ftl", import.meta.url), "utf8").split("### ghg:page-messages\n")[1] ?? "";

export const words = new FluentWords(CATALOGUE);

/** The shell of tests/fixtures/pages, its tree's landmark empty. */
export const SHELL = readFileSync(new URL("../../../tests/fixtures/pages/expected/index.html", import.meta.url), "utf8");

/** A document at `address`, empty but for a `main`. */
export function emptyDocument(address = "https://example.com/site/"): Document {
  const window = new Window({ url: address });
  window.document.write('<!doctype html><html lang="en"><body><main id="ghg_main"></main></body></html>');
  return window.document as unknown as Document;
}

/** `root`'s markup without its line feeds, which ghg 1 wrote between elements and the script does not; the pages compared hold none in their text. */
export function markupOf(root: Element): string {
  return root.innerHTML.replaceAll("\n", "");
}

/** A window at `address` on the shell of tests/fixtures/pages, its `main` the one committed at `path` below script/test/expected, as though the script had drawn it. */
export function drawnShell(path: string, address: string): Window {
  const window = new Window({ url: address });
  window.document.write(SHELL);
  const drawn = new Window({ url: address });
  drawn.document.write(readFileSync(new URL(`../expected/${path}`, import.meta.url), "utf8"));
  window.document.querySelector("main")!.outerHTML = drawn.document.querySelector("main")!.outerHTML;
  return window;
}

/** The `main` the script is expected to draw, committed at `path` below script/test/expected: ghg 1's, its links written as rule 235 writes them. */
export function expectedMain(path: string): Element {
  const window = new Window({ url: "https://example.com/site/" });
  window.document.write(readFileSync(new URL(`../expected/${path}`, import.meta.url), "utf8"));
  return window.document.querySelector("main")! as unknown as Element;
}
