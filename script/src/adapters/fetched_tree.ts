// Adapter: the tree source over the site's `assets/tree.json`, fetched over HTTP.

import { readTree } from "../policies/tree_state.ts";
import type { TreeItem, TreeSource } from "../ports/tree.ts";

export class FetchedTree implements TreeSource {
  private readonly address: URL;
  private readonly fetcher: typeof fetch;

  /** Chromium, through Playwright, refuses `fetch` called as a method of another object, so it is called on its own. */
  constructor(address: URL, fetcher: typeof fetch = (input, init) => fetch(input, init)) {
    this.address = address;
    this.fetcher = fetcher;
  }

  async load(): Promise<readonly TreeItem[] | null> {
    try {
      const response = await this.fetcher(this.address);
      return response.ok ? readTree(await response.json()) : null;
    } catch {
      return null;
    }
  }
}
