// Policy: the state a block starts in, and how the reader's choice is kept.

const EXPANDED = "expanded";
const COLLAPSED = "collapsed";

export function memoryKey(pagePath: string, anchor: string): string {
  return `gherkin-glass:${pagePath}#${anchor}`;
}

/** Where a background whose state the backgrounds' checkbox sets keeps it for the checked box; a page's path starts with `/`, so it never meets a key of `memoryKey`. */
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

/** Before what the reader chose; `expandedWhenUnchecked` is null for a block whose state does not follow the checkbox, and `inScenario` says whether the checked box expands it. */
export function formStart(written: boolean, expandedWhenUnchecked: boolean | null, inScenario: boolean, inline: boolean): boolean {
  if (expandedWhenUnchecked === null) {
    return written;
  }
  return inline ? inScenario : expandedWhenUnchecked;
}

/** The state the checkbox, checked when `inline`, sets a background to: the checked box expands those inside the scenarios and collapses those in their own place. */
export function switchedState(inScenario: boolean, inline: boolean): boolean {
  return inline === inScenario;
}
