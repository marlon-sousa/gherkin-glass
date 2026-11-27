// Tool: the pages a shell draws, from the descriptor it names, which the drawn pages and the browser checks both open.

import { pageByName, type PageName } from "../src/policies/address.ts";

/** What of a descriptor names the shell's pages. */
export interface PagesDescriptor {
  readonly features: readonly { readonly page: string }[];
  readonly tags: readonly { readonly page: string }[];
}

/** Every page of a shell of `descriptor`: its folders, the home page first, its features, the index of tags, its tags, the step sentences and the page of search results. */
export function shellPages(descriptor: PagesDescriptor): PageName[] {
  const features = descriptor.features.map((feature) => feature.page);
  const folders = new Set(features.flatMap(foldersAbove));
  const tags = descriptor.tags.map((tag) => tag.page);
  return [...["", ...folders].sort(), ...features, "tags/", ...tags, "steps"]
    .map(pageByName)
    .concat({ kind: "search", words: "" });
}

/** The folders `page` is in, each ending in `/`, outermost first. */
function foldersAbove(page: string): string[] {
  const parts = page.split("/").slice(0, -1);
  return parts.map((_, index) => `${parts.slice(0, index + 1).join("/")}/`);
}
