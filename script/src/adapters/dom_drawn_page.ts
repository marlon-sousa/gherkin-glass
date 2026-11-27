// Adapter: the drawn page port over the `main` of a shell, through the adapters of each part it finds there.

import type { DomCanvas } from "./dom_canvas.ts";
import { DomBackgrounds } from "./dom_backgrounds.ts";
import { DomNoteDialog } from "./dom_note_dialog.ts";
import { DomPage } from "./dom_page.ts";
import { DomSearchPage } from "./dom_search_page.ts";
import type { Backgrounds } from "../ports/backgrounds.ts";
import type { DrawnPage } from "../ports/drawn_page.ts";
import type { NoteDialog } from "../ports/note_dialog.ts";
import type { Page } from "../ports/page.ts";
import type { SearchPage } from "../ports/search_page.ts";

export class DomDrawnPage implements DrawnPage {
  private readonly document: Document;
  private readonly canvas: DomCanvas;

  constructor(document: Document, canvas: DomCanvas) {
    this.document = document;
    this.canvas = canvas;
  }

  blocks(): Page {
    return new DomPage(this.document, this.canvas.main);
  }

  backgrounds(): Backgrounds | null {
    return DomBackgrounds.of(this.document);
  }

  noteDialog(): NoteDialog | null {
    return DomNoteDialog.of(this.document);
  }

  searchPage(searched: string): SearchPage | null {
    return DomSearchPage.find(this.document, searched);
  }
}
