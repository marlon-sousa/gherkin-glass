// Policy: the folder tree's items, worked out from a descriptor and its features' objects, as docs/BEHAVIOUR.md, rule 89, lists them, and the items the current page is inside.

import { type PageName, targetAddress } from "./address.ts";
import { type ContentsEntry, contentsOf } from "./feature_view.ts";
import { heading } from "./heading_words.ts";
import { type LoadedFeature, type SiteFolder, folderName, siteFolders } from "./site_folders.ts";
import type { Descriptor } from "../ports/store.ts";
import type { TreeItem } from "../ports/tree.ts";
import type { Parts, Words } from "../ports/words.ts";

export function treeItems(words: Words, descriptor: Descriptor, features: readonly LoadedFeature[]): TreeItem[] {
  const folders = siteFolders(features);
  const itemsOf = (folder: SiteFolder | undefined): TreeItem[] =>
    folder === undefined
      ? []
      : [
          ...folder.folders.map((inner) => ({ text: folderName(inner), address: targetAddress({ kind: "folder", folder: inner }), items: itemsOf(folders.get(inner)) })),
          ...folder.features.map(({ entry, object }) => ({
            text: runsOf(heading(words, "feature", object.feature.name)),
            address: targetAddress({ kind: "feature", page: entry.page }),
            lang: object.lang,
            items: contentsOf(words, entry.page, object.feature).map(contentsItem),
          })),
        ];
  return [{ text: descriptor.title, address: targetAddress({ kind: "home" }) }, ...itemsOf(folders.get(""))];
}

/**
 * The addresses of the item of `page` and of every item it is inside, the
 * page's own first; empty when no item is the page's, as for a tag's page.
 */
export function pathToPage(items: readonly TreeItem[], page: PageName): string[] {
  const address = targetAddress(page);
  const pathTo = (inside: readonly TreeItem[]): string[] => {
    for (const item of inside) {
      if (item.address === address) {
        return [item.address];
      }
      const inner = pathTo(item.items ?? []);
      if (inner.length > 0) {
        return [...inner, item.address];
      }
    }
    return [];
  };
  return pathTo(items);
}

function contentsItem(entry: ContentsEntry): TreeItem {
  return { text: runsOf(entry.heading), address: entry.address, items: entry.entries.map(contentsItem) };
}

/** A tree item's text, as the tree port holds it: plain text whole, or runs, those at odd places isolated, the first plain. */
function runsOf(parts: Parts): string | string[] {
  const runs = parts.reduce<string[]>((done, part) => {
    const last = done.length - 1;
    if (part.isolated === (last % 2 === 1)) {
      return [...done.slice(0, last), `${done[last] ?? ""}${part.text}`];
    }
    return [...done, part.text];
  }, [""]);
  return runs.length === 1 ? (runs[0] ?? "") : runs;
}
