// What the tests of drawn pages share: the catalogue's words, a document to draw in, and a page ghg 1 wrote, made comparable with one the script draws.

import { readFileSync } from "node:fs";

import { Window } from "happy-dom";

import { FluentWords } from "../../src/adapters/fluent_words.ts";

export const CATALOGUE = readFileSync(new URL("../../../messages/en/gherkin-glass.ftl", import.meta.url), "utf8");

export const words = new FluentWords(CATALOGUE);

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

/** The `main` of the page ghg 1 wrote at `path` below tests/fixtures, the page `page`, each link to another page of the site written as rule 235 writes it. */
export function writtenMain(path: string, page: string): Element {
  const window = new Window({ url: "https://example.com/site/" });
  window.document.write(readFileSync(new URL(`../../../tests/fixtures/${path}`, import.meta.url), "utf8"));
  const main = window.document.querySelector("main")!;
  for (const link of main.querySelectorAll("a[href]")) {
    link.setAttribute("href", newAddress(link.getAttribute("href")!, page));
  }
  return main as unknown as Element;
}

/** A link of ghg 1, relative to the page `page`, as ghg 2 writes it: `#`, the page's name and its anchor. */
function newAddress(href: string, page: string): string {
  if (/^[a-z]+:/.test(href)) {
    return href;
  }
  if (href.startsWith("#")) {
    return `#${page}${href}`;
  }
  const [path = "", anchor] = href.split("#");
  const folders = page.split("/").slice(0, -1);
  for (const part of path.split("/")) {
    if (part === "..") {
      folders.pop();
    } else {
      folders.push(part);
    }
  }
  const name = folders.join("/").replace(/index\.html$/, "").replace(/\.html$/, "").replace(/^tags\/$/, "tags/");
  return anchor === undefined ? `#${name}` : `#${name}#${anchor}`;
}
