// Policy: whether the folder tree is shown, which of its items start expanded, how the reader's choices are kept, and what its data must be.

import type { TreeItem } from "../ports/tree.ts";

/** The media query of a window narrower than 48em, the width the stylesheet shows the button in place of the checkbox at, and stacks the tree above the content at. */
export const NARROW = "not all and (min-width: 48em)";

const SHOWN = "shown";
const HIDDEN = "hidden";
const EXPANDED = "expanded";
const COLLAPSED = "collapsed";

/** The keys hold a space, which a page's path, percent-encoded, never does. */
export function shownKey(sitePath: string): string {
  return `gherkin-glass:${sitePath} tree`;
}

export function itemKey(sitePath: string, address: string): string {
  return `gherkin-glass:${sitePath} tree ${address}`;
}

export function rememberedShown(shown: boolean): string {
  return shown ? SHOWN : HIDDEN;
}

export function rememberedExpanded(expanded: boolean): string {
  return expanded ? EXPANDED : COLLAPSED;
}

/** Whether the checkbox starts checked: the reader's choice wins over the configuration. */
export function startChecked(configured: boolean, kept: string | null): boolean {
  if (kept === SHOWN) {
    return true;
  }
  if (kept === HIDDEN) {
    return false;
  }
  return configured;
}

/** A narrow window shows the tree as its button says, any other as its checkbox does. */
export function treeShown(narrow: boolean, checked: boolean, open: boolean): boolean {
  return narrow ? open : checked;
}

/** An item the current page is inside is always expanded. */
export function startExpanded(onPath: boolean, kept: string | null): boolean {
  return onPath || kept === EXPANDED;
}

/**
 * The current page's address below the site's root, decoded, with the folder
 * page named; null when the page is not below it.
 */
export function pageAddress(sitePath: string, pagePath: string): string | null {
  const site = decoded(sitePath);
  const page = decoded(pagePath);
  if (!page.startsWith(site)) {
    return null;
  }
  const address = page.slice(site.length);
  return address === "" || address.endsWith("/") ? `${address}index.html` : address;
}

/** From the page at `address`, decoded, back to the site's root. */
export function rootFrom(address: string): string {
  return "../".repeat(address.split("/").length - 1);
}

/**
 * The addresses of the item of the page at `address`, decoded, and of every
 * item it is inside, the page's own first; empty when no item is the page's.
 */
export function pathTo(items: readonly TreeItem[], address: string): string[] {
  for (const item of items) {
    if (!item.address.includes("#") && decoded(item.address) === address) {
      return [item.address];
    }
    const inner = pathTo(item.items ?? [], address);
    if (inner.length > 0) {
      return [...inner, item.address];
    }
  }
  return [];
}

/** The items of `data`, or null when it is not a list of items. */
export function readTree(data: unknown): readonly TreeItem[] | null {
  return isItems(data) ? data : null;
}

function isItems(data: unknown): data is readonly TreeItem[] {
  return Array.isArray(data) && data.every(isItem);
}

function isItem(data: unknown): data is TreeItem {
  if (typeof data !== "object" || data === null) {
    return false;
  }
  const item = data as Record<string, unknown>;
  return (
    typeof item["text"] === "string" &&
    typeof item["address"] === "string" &&
    (item["lang"] === undefined || typeof item["lang"] === "string") &&
    (item["items"] === undefined || isItems(item["items"]))
  );
}

function decoded(path: string): string {
  try {
    return decodeURIComponent(path);
  } catch {
    return path;
  }
}
