// Policy: the state a block starts in, and how the reader's choice is kept.

const EXPANDED = "expanded";
const COLLAPSED = "collapsed";

export function memoryKey(pagePath: string, anchor: string): string {
  return `gherkin-glass:${pagePath}#${anchor}`;
}

/** Where a background whose steps a scenario starts with keeps its state for the inline form; a page's path starts with `/`, so it never meets a key of `memoryKey`. */
export function inlineMemoryKey(pagePath: string, anchor: string): string {
  return `gherkin-glass:inline:${pagePath}#${anchor}`;
}

export function remembered(expanded: boolean): string {
  return expanded ? EXPANDED : COLLAPSED;
}

/** A remembered state wins over the configured one; anything else kept is ignored. */
export function startState(configured: boolean, kept: string | null): boolean {
  if (kept === EXPANDED) {
    return true;
  }
  if (kept === COLLAPSED) {
    return false;
  }
  return configured;
}

/** Before what the reader chose; `expandedWhenLinked` is null for a block whose state does not follow the form. */
export function formStart(written: boolean, expandedWhenLinked: boolean | null, inline: boolean): boolean {
  if (expandedWhenLinked === null) {
    return written;
  }
  return !inline && expandedWhenLinked;
}
