// Service: the folder tree, shown or hidden as the reader chose, with the path to the current page expanded, remembered for the whole site.

import { itemKey, pathTo, rememberedExpanded, rememberedShown, shownKey, startExpanded, startShown } from "../policies/tree_state.ts";
import type { Memory } from "../ports/memory.ts";
import type { TreeSource, TreeSwitch, TreeView } from "../ports/tree.ts";

export interface TreePlace {
  /** The path of the site's root folder, ending in `/`. */
  readonly sitePath: string;
  /** The current page's address below it, decoded, or null. */
  readonly page: string | null;
  readonly narrow: boolean;
}

export async function startFolderTree(source: TreeSource, toggle: TreeSwitch, view: TreeView, memory: Memory, place: TreePlace): Promise<void> {
  const shown = startShown(toggle.configured(), memory.get(shownKey(place.sitePath)), place.narrow);
  toggle.setShown(shown);
  toggle.onChange((chosen) => {
    toggle.setShown(chosen);
    memory.set(shownKey(place.sitePath), rememberedShown(chosen));
  });
  const items = await source.load();
  if (items === null) {
    toggle.disable();
    return;
  }
  const path = place.page === null ? [] : pathTo(items, place.page);
  view.build(items, path[0] ?? null, (address) => startExpanded(path.includes(address), memory.get(itemKey(place.sitePath, address))));
  view.onToggle((address, expanded) => memory.set(itemKey(place.sitePath, address), rememberedExpanded(expanded)));
}
