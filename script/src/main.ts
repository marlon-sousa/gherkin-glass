// Composition root: the page's behaviour, wired to the browser, for a page written whole or for a shell whose pages the script draws.

import { DomAddress } from "./adapters/dom_address.ts";
import { DomCanvas } from "./adapters/dom_canvas.ts";
import { DomPages } from "./adapters/dom_pages.ts";
import { ElementBuilder } from "./adapters/element_builder.ts";
import { FetchedStore } from "./adapters/fetched_store.ts";
import { FluentWords } from "./adapters/fluent_words.ts";
import { DomBackgrounds } from "./adapters/dom_backgrounds.ts";
import { DomNoteDialog } from "./adapters/dom_note_dialog.ts";
import { DomPage } from "./adapters/dom_page.ts";
import { DomSearchBox } from "./adapters/dom_search_box.ts";
import { DomSearchPage } from "./adapters/dom_search_page.ts";
import { DomTheme } from "./adapters/dom_theme.ts";
import { DomTree } from "./adapters/dom_tree.ts";
import { DomTreeSwitch } from "./adapters/dom_tree_switch.ts";
import { FetchedDocuments } from "./adapters/fetched_documents.ts";
import { FetchedTree } from "./adapters/fetched_tree.ts";
import { LocalMemory } from "./adapters/local_memory.ts";
import { MediaWidth } from "./adapters/media_width.ts";
import { WindowTimer } from "./adapters/window_timer.ts";
import { targetAddress } from "./policies/address.ts";
import { siteRoot } from "./policies/background_form.ts";
import { NARROW, pageAddress } from "./policies/tree_state.ts";
import { startCollapsing } from "./services/collapsing.ts";
import { startFolderTree } from "./services/folder_tree.ts";
import { pageDrawing } from "./services/page_drawing.ts";
import { startNoteExplaining } from "./services/note_explaining.ts";
import { startNavigation } from "./services/navigation.ts";
import { startListing } from "./services/results_listing.ts";
import { startSearching } from "./services/searching.ts";
import { startShellTree } from "./services/shell_tree.ts";
import { startThemeSwitching } from "./services/theme_switching.ts";

const descriptorPath = document.documentElement.getAttribute("data-ghg-descriptor");
if (descriptorPath === null) {
  startWrittenPage();
} else {
  startShell(descriptorPath);
}

/** A page whose content ghg wrote whole. */
function startWrittenPage(): void {
  const memory = new LocalMemory();
  startThemeSwitching(new DomTheme(document), memory);
  const backgrounds = DomBackgrounds.of(document);
  startCollapsing(
    new DomPage(document),
    memory,
    document.location.pathname,
    backgrounds === null ? null : { backgrounds, siteRoot: siteRoot(import.meta.url) },
  );
  const noteDialog = DomNoteDialog.of(document);
  if (noteDialog !== null) {
    startNoteExplaining(noteDialog);
  }

  // The script is in the site's `assets/` folder.
  const site = new URL("../", import.meta.url);
  const sitePath = site.pathname;
  const page = pageAddress(sitePath, document.location.pathname);
  void startFolderTree(
    new FetchedTree(new URL("tree.json", import.meta.url)),
    new DomTreeSwitch(document),
    new DomTree(document, site, page),
    new MediaWidth(window.matchMedia(NARROW)),
    memory,
    { sitePath, page },
  );
  const searchDocuments = new FetchedDocuments(new URL("search.json", import.meta.url));
  // The page of search results fills the search box's field before its first wait, so the box,
  // started after it, begins with those words.
  const searchPage = DomSearchPage.find(document);
  if (searchPage !== null) {
    void startListing(searchPage, searchDocuments);
  }
  const searchBox = DomSearchBox.find(document);
  if (searchBox !== null) {
    startSearching(searchBox, searchDocuments, new WindowTimer());
  }
}

/** A shell, whose pages the script draws from the site's data, found at `descriptorPath` in the store. */
function startShell(descriptorPath: string): void {
  const make = new ElementBuilder(document);
  const canvas = DomCanvas.of(make);
  if (canvas === null) {
    return;
  }
  const memory = new LocalMemory();
  startThemeSwitching(new DomTheme(document), memory);
  // The script is in `assets/<ghg>/` at the output folder's root, beside `data/`.
  const root = new URL("../../", import.meta.url);
  const words = new FluentWords(document.getElementById("ghg_messages")?.textContent ?? "");
  const store = new FetchedStore(new URL("data/", root), descriptorPath);
  const shellPath = document.location.pathname.replace(/index\.html$/, "");
  const tree = startShellTree({
    store,
    words,
    toggle: new DomTreeSwitch(document),
    view: DomTree.inShell(document),
    width: new MediaWidth(window.matchMedia(NARROW)),
    memory,
    sitePath: shellPath,
  });
  void startNavigation({
    canvas,
    address: new DomAddress(window),
    timer: new WindowTimer(),
    draw: pageDrawing({ store, words, pages: new DomPages(make, canvas, words), canvas }),
    behave: (page) => {
      tree.show(page);
      const backgrounds = DomBackgrounds.of(document);
      const reveal = startCollapsing(new DomPage(document, canvas.main, false), memory, `${shellPath}${targetAddress(page)}`, backgrounds && { backgrounds, siteRoot: root.pathname });
      const noteDialog = DomNoteDialog.of(document);
      if (noteDialog !== null) {
        startNoteExplaining(noteDialog);
      }
      return reveal;
    },
    failure: words.text("page-not-loaded"),
    onFailure: () => tree.disable(),
  });
}
