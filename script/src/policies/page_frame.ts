// Policy: what surrounds a page's content in its shell, its title, its language and its breadcrumb, which on a version start at the versions page and name the version.

import { type PageName, targetAddress } from "./address.ts";
import { plain } from "./heading_words.ts";
import type { Descriptor } from "../ports/store.ts";
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

/** The page that says nothing is at an address that names no page of the shell. */
export function missingFrame(words: Words, siteTitle: string): PageFrame {
  const heading = words.text("page-missing");
  return { title: words.text("page-missing-title", { heading, site: siteTitle }), lang: "en", crumbs: [homeCrumb(siteTitle)] };
}

/** The versions page, above every version's shell. */
const VERSIONS_PAGE = "../index.html";

/** The version `descriptor` is of, with what it is when it is a development version or a pre-release; null for a site without versions and a preview. */
export function versionWords(words: Words, descriptor: Descriptor): string | null {
  const name = descriptor.version;
  if (name === null) {
    return null;
  }
  switch (descriptor.version_kind) {
    case null:
      return null;
    case "development":
      return words.text("versions-development-version", { name });
    case "pre-release":
      return words.text("versions-pre-release-version", { name });
    case "release":
      return name;
  }
}

/** `title`, ended with `version` on a page of a version. */
export function versionedTitle(words: Words, version: string | null, title: string): string {
  return version === null ? title : words.text("version-page-title", { title, version });
}

/** `frame` on a page of the version worded `version`: its title ends with the version, and its breadcrumb starts with the versions page, then the version's home page, which reads the version. */
export function versionedFrame(words: Words, version: string | null, frame: PageFrame): PageFrame {
  if (version === null) {
    return frame;
  }
  const [home, ...rest] = frame.crumbs;
  const versions: Crumb = { address: VERSIONS_PAGE, words: plain(words.text("versions-title")), lang: null };
  const crumbs = home === undefined ? [versions] : [versions, { ...home, words: plain(version), lang: null }, ...rest];
  return { ...frame, title: versionedTitle(words, version, frame.title), crumbs };
}
