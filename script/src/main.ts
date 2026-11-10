// Composition root: the page's behaviour, wired to the browser.

import { DomPage } from "./adapters/dom_page.ts";
import { DomTree, DomTreeSwitch } from "./adapters/dom_tree.ts";
import { FetchedTree } from "./adapters/fetched_tree.ts";
import { LocalMemory } from "./adapters/local_memory.ts";
import { NARROW, pageAddress } from "./policies/tree_state.ts";
import { startCollapsing } from "./services/collapsing.ts";
import { startFolderTree } from "./services/folder_tree.ts";

const memory = new LocalMemory();
startCollapsing(new DomPage(document), memory, document.location.pathname);

// The script is in the site's `assets/` folder.
const site = new URL("../", import.meta.url);
const sitePath = site.pathname;
const page = pageAddress(sitePath, document.location.pathname);
void startFolderTree(new FetchedTree(new URL("tree.json", import.meta.url)), new DomTreeSwitch(document), new DomTree(document, site, page), memory, {
  sitePath,
  page,
  narrow: window.matchMedia(NARROW).matches,
});
