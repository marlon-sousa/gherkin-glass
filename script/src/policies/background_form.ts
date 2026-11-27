// Policy: whether a page's backgrounds' checkbox is checked, and how the reader's choice is kept for the whole site; `inline` and `linked` are the values kept since the box chose between two forms.

const INLINE = "inline";
const LINKED = "linked";

/** Holds no `#`, which every block's key holds, so that the two never meet. */
export function formKey(siteRoot: string): string {
  return `gherkin-glass:${siteRoot}:backgrounds`;
}

export function rememberedForm(inline: boolean): string {
  return inline ? INLINE : LINKED;
}

/** A remembered state of the box wins over the configured one; anything else kept is ignored. */
export function startForm(configured: boolean, kept: string | null): boolean {
  if (kept === INLINE) {
    return true;
  }
  if (kept === LINKED) {
    return false;
  }
  return configured;
}
