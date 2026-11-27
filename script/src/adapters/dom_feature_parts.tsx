// Adapter: the markup of a feature's text on a page, its tags, description, steps, tables, doc strings and the note of a keyword, in the elements and classes the stylesheet styles.

import type { Markup } from "./jsx/jsx_runtime.ts";
import { MessageParts } from "./jsx/message_parts.tsx";
import type { StepView, TableView, TagView } from "../policies/block_views.ts";
import type { NoteWords } from "../policies/heading_words.ts";
import type { DocString, Part } from "../ports/store.ts";

export function tagList(tags: readonly TagView[]): Markup | null {
  if (tags.length === 0) {
    return null;
  }
  return (
    <ul class="ghg-tags">
      {tags.map((tag) => (
        <li>
          <a href={tag.address}>{tag.text}</a>
          {tag.link && " "}
          {tag.link && (
            <a class="ghg-tag-link" href={tag.link.address}>
              {tag.link.text}
            </a>
          )}
        </li>
      ))}
    </ul>
  );
}

export function descriptionOf(lines: readonly string[]): Markup | null {
  if (lines.length === 0) {
    return null;
  }
  return (
    <div class="ghg-description">
      <p>{lines.map((line, place) => (place === 0 ? line : [<br />, "\n", line]))}</p>
    </div>
  );
}

export function noteOf(note: NoteWords | null): Markup | null {
  if (note === null) {
    return null;
  }
  return (
    <p class="ghg-keyword-note" role="note">
      {note.before}
      <button type="button" class="ghg-note-opener" aria-haspopup="dialog" aria-controls="ghg_note_dialog">
        <MessageParts parts={note.opener} />
      </button>
      {note.after}
    </p>
  );
}

export function tableOf(table: TableView): Markup {
  return (
    <table class="ghg-table">
      <thead>
        <tr>
          {table.header.map((cell) => (
            <th scope="col">{cell}</th>
          ))}
        </tr>
      </thead>
      {table.rows.length > 0 && (
        <tbody>
          {table.rows.map((row) => (
            <tr>
              {row.map((cell) => (
                <td>{cell}</td>
              ))}
            </tr>
          ))}
        </tbody>
      )}
    </table>
  );
}

function docStringOf(doc: DocString): Markup {
  if (doc.media_type === null) {
    return <pre class="ghg-doc-string">{doc.content}</pre>;
  }
  return (
    <figure class="ghg-doc-string">
      <figcaption>{doc.media_type}</figcaption>
      <pre>{doc.content}</pre>
    </figure>
  );
}

export function stepList(steps: readonly StepView[]): Markup | null {
  if (steps.length === 0) {
    return null;
  }
  return (
    <ol class="ghg-steps">
      {steps.map((step) => (
        <li>
          <span class="ghg-keyword">{step.keyword}</span>
          {step.parts.map(partOf)}
          {step.table && ["\n", tableOf(step.table)]}
          {step.docString && ["\n", docStringOf(step.docString)]}
        </li>
      ))}
    </ol>
  );
}

function partOf(part: Part): Markup | string {
  switch (part.kind) {
    case "text":
      return part.text;
    case "parameter":
      return <span class="ghg-parameter">{part.parts.map((inner) => (inner.kind === "placeholder" ? partOf(inner) : inner.text))}</span>;
    case "number":
    case "placeholder":
      return <span class={`ghg-${part.kind}`}>{part.text}</span>;
  }
}
