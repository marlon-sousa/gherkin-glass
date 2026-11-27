// Adapter: the link a click on the page follows in the same tab, for the adapters that catch a link as it is followed.

/** Null for a click that opens another tab or window, that a script stopped, or that is on no link. */
export function followedLink(event: MouseEvent): HTMLAnchorElement | null {
  if (event.defaultPrevented || event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) {
    return null;
  }
  // The window's own classes are not the script's globals in a test, so the target is known by what it can do.
  const target = event.target as Element | null;
  return typeof target?.closest === "function" ? target.closest<HTMLAnchorElement>("a[href]") : null;
}
