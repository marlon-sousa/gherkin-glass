// Policy: the state a block starts in, and how the reader's choice is kept.

const EXPANDED = "expanded";
const COLLAPSED = "collapsed";

export function memoryKey(pagePath: string, anchor: string): string {
  return `gherkin-glass:${pagePath}#${anchor}`;
}

/**
 * Where a background whose steps a scenario starts with keeps its state while the page shows those steps inside each scenario.
 * `memoryKey` keeps its state for the linked form, and every other block's.
 */
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

/**
 * The state a block starts in with the form shown, before what the reader chose: a background whose steps a scenario starts with
 * is collapsed with the inline form and as the page says with the linked one; any other block is as the page was written.
 */
export function formStart(written: boolean, expandedWhenLinked: boolean | null, inline: boolean): boolean {
  if (expandedWhenLinked === null) {
    return written;
  }
  return !inline && expandedWhenLinked;
}
