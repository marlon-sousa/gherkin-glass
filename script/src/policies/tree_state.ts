// Policy: whether the folder tree is shown, whether a resize hides what has the focus, which of its items start expanded, and how the reader's choices are kept.

import type { FocusPlace } from "../ports/tree.ts";

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

/**
 * Whether a resize to a `narrow` window or not hides what had the focus,
 * `place`, which would drop the focus to the body: the tree, once it is no
 * longer `shown`, the checkbox on a narrow window, or the button on a wide one.
 */
export function focusHidden(place: FocusPlace, narrow: boolean, shown: boolean): boolean {
  switch (place) {
    case "tree":
      return !shown;
    case "checkbox":
      return narrow;
    case "button":
      return !narrow;
    case null:
      return false;
  }
}

/** An item the current page is inside is always expanded. */
export function startExpanded(onPath: boolean, kept: string | null): boolean {
  return onPath || kept === EXPANDED;
}
