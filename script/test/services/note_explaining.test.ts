// The service of src/services/note_explaining.ts, with a fake port: docs/BEHAVIOUR.md, rule 198.

import assert from "node:assert/strict";
import { test } from "node:test";

import type { NoteDialog, Opener } from "../../src/ports/note_dialog.ts";
import { startNoteExplaining } from "../../src/services/note_explaining.ts";

class FakeDialog implements NoteDialog {
  opened = 0;
  press: (opener: Opener) => void = () => {};
  close: () => void = () => {};

  onOpen(handler: (opener: Opener) => void): void {
    this.press = handler;
  }

  open(): void {
    this.opened += 1;
  }

  onClose(handler: () => void): void {
    this.close = handler;
  }
}

class FakeOpener implements Opener {
  focused = 0;

  focus(): void {
    this.focused += 1;
  }
}

test("a note's button opens the dialog, and closing it gives the focus back to that button", () => {
  const dialog = new FakeDialog();
  startNoteExplaining(dialog);
  const first = new FakeOpener();
  const second = new FakeOpener();
  dialog.press(first);
  assert.equal(dialog.opened, 1);
  assert.equal(first.focused, 0);
  dialog.close();
  assert.equal(first.focused, 1);
  dialog.press(second);
  dialog.close();
  assert.deepEqual([first.focused, second.focused, dialog.opened], [1, 1, 2]);
});

test("a dialog closed again gives the focus to nothing more", () => {
  const dialog = new FakeDialog();
  startNoteExplaining(dialog);
  const opener = new FakeOpener();
  dialog.press(opener);
  dialog.close();
  dialog.close();
  assert.equal(opener.focused, 1);
});
