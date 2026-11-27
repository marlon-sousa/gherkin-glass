// Adapter: the note dialog port over the page's dialog and the buttons in its notes, as dom_feature_page.tsx and dom_feature_parts.tsx draw them.

import type { NoteDialog, Opener } from "../ports/note_dialog.ts";

export class DomNoteDialog implements NoteDialog {
  private constructor(
    private readonly dialog: HTMLDialogElement,
    private readonly openers: HTMLButtonElement[],
  ) {}

  /** Null on a page without the dialog, where no block's file writes another keyword. */
  static of(document: Document): DomNoteDialog | null {
    const dialog = document.querySelector<HTMLDialogElement>("dialog#ghg_note_dialog");
    return dialog ? new DomNoteDialog(dialog, [...document.querySelectorAll<HTMLButtonElement>("button.ghg-note-opener")]) : null;
  }

  onOpen(handler: (opener: Opener) => void): void {
    this.openers.forEach((button) => button.addEventListener("click", () => handler(button)));
  }

  open(): void {
    this.dialog.showModal();
    this.dialog.focus();
  }

  onClose(handler: () => void): void {
    this.dialog.addEventListener("close", handler);
  }
}
