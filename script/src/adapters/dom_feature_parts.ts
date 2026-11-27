// Adapter: the elements of a feature's text on a page, its tags, description, steps, tables, doc strings and the note of a keyword, as today's templates in src/policies/rendering write them.

import type { ElementBuilder } from "./element_builder.ts";
import type { StepView, TableView, TagView } from "../policies/block_views.ts";
import type { NoteWords } from "../policies/heading_words.ts";
import type { DocString, Part } from "../ports/store.ts";

export function tagList(make: ElementBuilder, tags: readonly TagView[]): HTMLElement | null {
  if (tags.length === 0) {
    return null;
  }
  return make.make(
    "ul",
    { class: "ghg-tags" },
    ...tags.map((tag) =>
      make.make("li", {}, make.make("a", { href: tag.address }, tag.text), tag.link && " ", tag.link && make.make("a", { class: "ghg-tag-link", href: tag.link.address }, tag.link.text)),
    ),
  );
}

export function descriptionOf(make: ElementBuilder, lines: readonly string[]): HTMLElement | null {
  if (lines.length === 0) {
    return null;
  }
  const paragraph = make.make("p");
  lines.forEach((line, place) => {
    if (place > 0) {
      paragraph.append(make.make("br"), "\n");
    }
    paragraph.append(line);
  });
  return make.make("div", { class: "ghg-description" }, paragraph);
}

export function noteOf(make: ElementBuilder, note: NoteWords | null): HTMLElement | null {
  if (note === null) {
    return null;
  }
  const opener = make.make("button", { type: "button", class: "ghg-note-opener", "aria-haspopup": "dialog", "aria-controls": "ghg_note_dialog" }, ...make.words(note.opener));
  return make.make("p", { class: "ghg-keyword-note", role: "note" }, note.before, opener, note.after);
}

export function tableOf(make: ElementBuilder, table: TableView): HTMLTableElement {
  const head = make.make("thead", {}, make.make("tr", {}, ...table.header.map((cell) => make.make("th", { scope: "col" }, cell))));
  const body = table.rows.length === 0 ? null : make.make("tbody", {}, ...table.rows.map((row) => make.make("tr", {}, ...row.map((cell) => make.make("td", {}, cell)))));
  return make.make("table", { class: "ghg-table" }, head, body);
}

function docStringOf(make: ElementBuilder, doc: DocString): HTMLElement {
  if (doc.media_type === null) {
    return make.make("pre", { class: "ghg-doc-string" }, doc.content);
  }
  return make.make("figure", { class: "ghg-doc-string" }, make.make("figcaption", {}, doc.media_type), make.make("pre", {}, doc.content));
}

export function stepList(make: ElementBuilder, steps: readonly StepView[]): HTMLElement | null {
  if (steps.length === 0) {
    return null;
  }
  return make.make("ol", { class: "ghg-steps" }, ...steps.map((step) => stepItem(make, step)));
}

function stepItem(make: ElementBuilder, step: StepView): HTMLElement {
  return make.make(
    "li",
    {},
    make.make("span", { class: "ghg-keyword" }, step.keyword),
    ...step.parts.map((part) => partOf(make, part)),
    step.table && "\n",
    step.table && tableOf(make, step.table),
    step.docString && "\n",
    step.docString && docStringOf(make, step.docString),
  );
}

function partOf(make: ElementBuilder, part: Part): Node | string {
  switch (part.kind) {
    case "text":
      return part.text;
    case "parameter":
      return make.make("span", { class: "ghg-parameter" }, ...part.parts.map((inner) => (inner.kind === "placeholder" ? partOf(make, inner) : inner.text)));
    case "number":
    case "placeholder":
      return make.make("span", { class: `ghg-${part.kind}` }, part.text);
  }
}
