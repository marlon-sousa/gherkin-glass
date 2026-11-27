// Composition root: the behaviour of a shell, whose pages the script draws, or of the versions page, whose content ghg wrote whole, wired to the browser.

import { DomAddress } from "./adapters/dom_address.ts";
import { DomCanvas } from "./adapters/dom_canvas.ts";
import { DomPages } from "./adapters/dom_pages.ts";
import { ElementBuilder } from "./adapters/element_builder.ts";
import { FetchedStore } from "./adapters/fetched_store.ts";
import { FluentWords } from "./adapters/fluent_words.ts";
import { DomDrawnPage } from "./adapters/dom_drawn_page.ts";
import { DomPage } from "./adapters/dom_page.ts";
import { DomSearchBox } from "./adapters/dom_search_box.ts";
import { DomTheme } from "./adapters/dom_theme.ts";
import { DomTree } from "./adapters/dom_tree.ts";
import { DomTreeSwitch } from "./adapters/dom_tree_switch.ts";
import { LocalMemory } from "./adapters/local_memory.ts";
import { MediaWidth } from "./adapters/media_width.ts";
import { WindowTimer } from "./adapters/window_timer.ts";
import { NARROW } from "./policies/tree_state.ts";
import { storeDocuments } from "./services/loaded_features.ts";
import { startCollapsing } from "./services/collapsing.ts";
import { pageDrawing } from "./services/page_drawing.ts";
import { startNavigation } from "./services/navigation.ts";
import { startSearching } from "./services/searching.ts";
import { shellBehaviour } from "./services/shell_behaviour.ts";
import { startShellTree } from "./services/shell_tree.ts";
import { startThemeSwitching } from "./services/theme_switching.ts";

const descriptorPath = document.documentElement.getAttribute("data-ghg-descriptor");
if (descriptorPath === null) {
  startVersionsPage();
} else {
  startShell(descriptorPath);
}

/** The versions page: its theme, and its pre-releases, a block that collapses. */
function startVersionsPage(): void {
  const memory = new LocalMemory();
  startThemeSwitching(new DomTheme(document), memory);
  startCollapsing(new DomPage(document), memory, document.location.pathname, null);
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
    view: new DomTree(document),
    width: new MediaWidth(window.matchMedia(NARROW)),
    memory,
    sitePath: shellPath,
  });
  const documents = storeDocuments(store);
  const searchBox = DomSearchBox.find(document);
  const fillSearch = searchBox === null ? null : startSearching(searchBox, documents, new WindowTimer());
  void startNavigation({
    canvas,
    address: new DomAddress(window),
    timer: new WindowTimer(),
    draw: pageDrawing({ store, words, pages: new DomPages(make, canvas, words), canvas }),
    behave: shellBehaviour({ drawn: new DomDrawnPage(document, canvas), tree, fillSearch, documents, memory, shellPath }),
    failure: words.text("page-not-loaded"),
    onFailure: () => tree.disable(),
  });
}
