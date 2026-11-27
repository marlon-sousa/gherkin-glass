// Port: the parts of the page a shell has just drawn that the reader uses, found in its `main` once it is drawn.

import type { Backgrounds } from "./backgrounds.ts";
import type { NoteDialog } from "./note_dialog.ts";
import type { Page } from "./page.ts";
import type { SearchPage } from "./search_page.ts";

export interface DrawnPage {
  blocks(): Page;
  /** Null on a page without the backgrounds' checkbox. */
  backgrounds(): Backgrounds | null;
  /** Null on a page without the dialog that explains a keyword's note. */
  noteDialog(): NoteDialog | null;
  /** Null on a page that is not the page of search results; `searched` is the words its address searches for. */
  searchPage(searched: string): SearchPage | null;
}
