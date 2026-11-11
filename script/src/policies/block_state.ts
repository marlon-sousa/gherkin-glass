// Policy: the state a block starts in, and how the reader's choice is kept.

const EXPANDED = "expanded";
const COLLAPSED = "collapsed";

export function memoryKey(pagePath: string, anchor: string): string {
  return `gherkin-glass:${pagePath}#${anchor}`;
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
