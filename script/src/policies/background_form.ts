// Policy: the form a page shows its backgrounds in, and how the reader's choice is kept for the whole site.

const INLINE = "inline";
const LINKED = "linked";

/** The path of the site's root folder, from the address of the script, which is `assets/gherkin-glass.js` in it. */
export function siteRoot(scriptAddress: string): string {
  return new URL("..", scriptAddress).pathname;
}

/** Holds no `#`, which every block's key holds, so that the two never meet. */
export function formKey(siteRoot: string): string {
  return `gherkin-glass:${siteRoot}:backgrounds`;
}

export function rememberedForm(inline: boolean): string {
  return inline ? INLINE : LINKED;
}

/** A remembered form wins over the configured one; anything else kept is ignored. */
export function startForm(configured: boolean, kept: string | null): boolean {
  if (kept === INLINE) {
    return true;
  }
  if (kept === LINKED) {
    return false;
  }
  return configured;
}
