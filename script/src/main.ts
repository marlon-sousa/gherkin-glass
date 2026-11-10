// Composition root: the page's behaviour, wired to the browser.

import { DomPage } from "./adapters/dom_page.ts";
import { LocalMemory } from "./adapters/local_memory.ts";
import { startCollapsing } from "./services/collapsing.ts";

startCollapsing(new DomPage(document), new LocalMemory(), document.location.pathname);
