// Policy: what surrounds a page's content in its shell, its title, its language and its breadcrumb, as docs/BEHAVIOUR.md, rules 81, 85 and 239, give them.

import { type PageName, targetAddress } from "./address.ts";
import { plain } from "./heading_words.ts";
import type { Parts, Words } from "../ports/words.ts";

export interface Crumb {
  readonly address: string;
  readonly words: Parts;
  /** The `lang` of its feature text, when it is not the page's. */
  readonly lang: string | null;
}

export interface PageFrame {
  readonly title: string;
  readonly lang: string;
  /** From the home page in, the page's own last. */
  readonly crumbs: readonly Crumb[];
}

const HOME: PageName = { kind: "home" };

/** The link to the home page, which reads the site's title. */
export function homeCrumb(siteTitle: string): Crumb {
  return { address: targetAddress(HOME), words: plain(siteTitle), lang: null };
}

/** The crumbs of each folder above the page `name`, such as `billing/` and `billing/archive/` above `billing/archive/refunds`. */
export function folderCrumbs(name: string): Crumb[] {
  const folders = name.split("/").slice(0, -1);
  return folders.map((folder, place) => ({
    address: targetAddress({ kind: "folder", folder: `${folders.slice(0, place + 1).join("/")}/` }),
    words: plain(folder),
    lang: null,
  }));
}

/** The frame of a page named `page`, whose own crumb reads `words`, below `above`. */
export function frameOf(siteTitle: string, title: string, lang: string, above: readonly Crumb[], page: PageName, words: Parts): PageFrame {
  return { title, lang, crumbs: [homeCrumb(siteTitle), ...above, { address: targetAddress(page), words, lang: null }] };
}

/** The page of rule 239, for an address that names no page of the shell. */
export function missingFrame(words: Words, siteTitle: string): PageFrame {
  const heading = words.text("page-missing");
  return { title: words.text("page-missing-title", { heading, site: siteTitle }), lang: "en", crumbs: [homeCrumb(siteTitle)] };
}
