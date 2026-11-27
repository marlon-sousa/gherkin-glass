// Service: what the page a shell has just drawn does, its item marked in the folder tree, its search box filled and its results listed, its blocks' buttons and its keywords' notes.

import { type PageName, targetAddress } from "../policies/address.ts";
import { SHELL_ADDRESSES } from "../policies/search_index.ts";
import type { Documents } from "../ports/documents.ts";
import type { DrawnPage } from "../ports/drawn_page.ts";
import type { Memory } from "../ports/memory.ts";
import { startCollapsing } from "./collapsing.ts";
import type { PageBehaviour } from "./navigation.ts";
import { startNoteExplaining } from "./note_explaining.ts";
import { startListing } from "./results_listing.ts";
import type { SearchFilling } from "./searching.ts";
import type { ShellTree } from "./shell_tree.ts";

export interface ShellBehaviourParts {
  readonly drawn: DrawnPage;
  readonly tree: ShellTree;
  /** Null for a shell without a search box. */
  readonly fillSearch: SearchFilling | null;
  readonly documents: Documents;
  readonly memory: Memory;
  /** The path of the shell's folder, ending in `/`, under which the reader's choices are kept. */
  readonly shellPath: string;
  /** The path of the output folder's root, under which the choice of the backgrounds' form is kept. */
  readonly siteRoot: string;
}

export function shellBehaviour(parts: ShellBehaviourParts): PageBehaviour {
  return (page: PageName) => {
    parts.tree.show(page);
    if (page.kind === "search") {
      parts.fillSearch?.(page.words);
      const searchPage = parts.drawn.searchPage(page.words);
      if (searchPage !== null) {
        void startListing(searchPage, parts.documents, SHELL_ADDRESSES);
      }
    }
    const backgrounds = parts.drawn.backgrounds();
    const reveal = startCollapsing(parts.drawn.blocks(), parts.memory, `${parts.shellPath}${targetAddress(page)}`, backgrounds && { backgrounds, siteRoot: parts.siteRoot });
    const noteDialog = parts.drawn.noteDialog();
    if (noteDialog !== null) {
      startNoteExplaining(noteDialog);
    }
    return reveal;
  };
}
