// Composition root: the page's behaviour, wired to the browser.

import { DomBackgrounds } from "./adapters/dom_backgrounds.ts";
import { DomPage } from "./adapters/dom_page.ts";
import { DomSearchBox } from "./adapters/dom_search_box.ts";
import { DomTheme } from "./adapters/dom_theme.ts";
import { DomTree, DomTreeSwitch } from "./adapters/dom_tree.ts";
import { FetchedDocuments } from "./adapters/fetched_documents.ts";
import { FetchedTree } from "./adapters/fetched_tree.ts";
import { LocalMemory } from "./adapters/local_memory.ts";
import { WindowTimer } from "./adapters/window_timer.ts";
import { siteRoot } from "./policies/background_form.ts";
import { NARROW, pageAddress } from "./policies/tree_state.ts";
import { startCollapsing } from "./services/collapsing.ts";
import { startFolderTree } from "./services/folder_tree.ts";
import { startSearching } from "./services/searching.ts";
import { startThemeSwitching } from "./services/theme_switching.ts";

const memory = new LocalMemory();
startThemeSwitching(new DomTheme(document), memory);
const backgrounds = DomBackgrounds.of(document);
startCollapsing(
  new DomPage(document),
  memory,
  document.location.pathname,
  backgrounds === null ? null : { backgrounds, siteRoot: siteRoot(import.meta.url) },
);

// The script is in the site's `assets/` folder.
const site = new URL("../", import.meta.url);
const sitePath = site.pathname;
const page = pageAddress(sitePath, document.location.pathname);
void startFolderTree(new FetchedTree(new URL("tree.json", import.meta.url)), new DomTreeSwitch(document), new DomTree(document, site, page), memory, {
  sitePath,
  page,
  narrow: window.matchMedia(NARROW).matches,
});
const searchBox = DomSearchBox.find(document);
if (searchBox !== null) {
  startSearching(searchBox, new FetchedDocuments(new URL("search.json", import.meta.url)), new WindowTimer());
}
