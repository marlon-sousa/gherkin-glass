// The adapter of src/adapters/dom_note_dialog.ts, on pages gherkin-glass wrote: tests/fixtures/pages/expected, with the service behind it: docs/BEHAVIOUR.md, rules 196 to 198.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { Window } from "happy-dom";

import { DomNoteDialog } from "../../src/adapters/dom_note_dialog.ts";
import { startNoteExplaining } from "../../src/services/note_explaining.ts";

const SITE = new URL("../../../tests/fixtures/pages/expected/", import.meta.url);

function open(page: string): Window {
  const window = new Window({ url: `https://example.com/site/${page}` });
  window.document.write(readFileSync(new URL(page, SITE), "utf8"));
  return window;
}

test("a page whose files write only their kinds' words has no dialog", () => {
  const window = open("outlines.html");
  assert.equal(DomNoteDialog.of(window.document as unknown as Document), null);
});

test("a note's button opens the dialog with the focus on it, and closing it gives the focus back", () => {
  const window = open("kinds.html");
  const document = window.document as unknown as Document;
  const notes = DomNoteDialog.of(document);
  assert.ok(notes !== null);
  startNoteExplaining(notes);
  const dialog = document.querySelector<HTMLDialogElement>("dialog#ghg_note_dialog");
  const openers = [...document.querySelectorAll<HTMLButtonElement>("button.ghg-note-opener")];
  assert.ok(dialog !== null && openers.length > 1);
  assert.equal(dialog.open, false);
  const opener = openers[1];
  opener.focus();
  opener.click();
  assert.equal(dialog.open, true);
  assert.ok(document.activeElement === dialog);
  dialog.querySelector("button")?.focus();
  dialog.close();
  assert.equal(dialog.open, false);
  assert.ok(document.activeElement === opener);
});
