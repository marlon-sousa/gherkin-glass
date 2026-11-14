// Port: the dialog that explains a note of the keyword a block's file writes, and the buttons in the notes that open it.

/** What had the focus when the dialog opened, to give it back to. */
export interface Opener {
  focus(): void;
}

export interface NoteDialog {
  /** The reader pressed the button of a note. */
  onOpen(handler: (opener: Opener) => void): void;
  /** Shows the dialog, modal, with the focus on the dialog itself. */
  open(): void;
  /** The dialog closed, by its button or by Escape. */
  onClose(handler: () => void): void;
}
