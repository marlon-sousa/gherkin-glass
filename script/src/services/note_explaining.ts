// Service: the dialog that explains a note of the keyword a block's file writes, opened from the note, which gets the focus back once it closes.

import type { NoteDialog, Opener } from "../ports/note_dialog.ts";

export function startNoteExplaining(dialog: NoteDialog): void {
  let opener: Opener | null = null;
  dialog.onOpen((pressed) => {
    opener = pressed;
    dialog.open();
  });
  dialog.onClose(() => {
    opener?.focus();
    opener = null;
  });
}
