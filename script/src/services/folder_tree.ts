// Service: the folder tree, shown or hidden as the reader chose with the checkbox on a wide window and with the button on a narrow one, with the path to the current page expanded, the checkbox's choice remembered for the whole site.

import { itemKey, pathTo, rememberedExpanded, rememberedShown, shownKey, startChecked, startExpanded, treeShown } from "../policies/tree_state.ts";
import type { Memory } from "../ports/memory.ts";
import type { TreeSource, TreeSwitch, TreeView } from "../ports/tree.ts";
import type { WindowWidth } from "../ports/width.ts";

export interface TreePlace {
  /** The path of the site's root folder, ending in `/`. */
  readonly sitePath: string;
  /** The current page's address below it, decoded, or null. */
  readonly page: string | null;
}

export async function startFolderTree(source: TreeSource, toggle: TreeSwitch, view: TreeView, width: WindowWidth, memory: Memory, place: TreePlace): Promise<void> {
  let checked = startChecked(toggle.configured(), memory.get(shownKey(place.sitePath)));
  // The button's state is never remembered.
  let open = false;
  let narrow = width.narrow();
  let usable = true;
  const show = (): void => {
    toggle.setChecked(usable && checked);
    toggle.setOpen(usable && open);
    toggle.setShown(usable && treeShown(narrow, checked, open));
  };
  show();
  toggle.onChange((chosen) => {
    checked = chosen;
    memory.set(shownKey(place.sitePath), rememberedShown(chosen));
    show();
  });
  toggle.onPress(() => {
    open = !open;
    show();
  });
  toggle.onEscape(() => {
    if (narrow && open) {
      open = false;
      show();
      toggle.focusButton();
    }
  });
  width.onChange((now) => {
    const focusInTree = toggle.holdsFocus();
    narrow = now;
    open = false;
    show();
    // A tree hidden with the focus inside it would drop the focus to the body.
    if (focusInTree && !treeShown(narrow, checked, open)) {
      if (narrow) {
        toggle.focusButton();
      } else {
        toggle.focusCheckbox();
      }
    }
  });
  const items = await source.load();
  if (items === null) {
    usable = false;
    toggle.disable();
    show();
    return;
  }
  const path = place.page === null ? [] : pathTo(items, place.page);
  view.build(items, path[0] ?? null, (address) => startExpanded(path.includes(address), memory.get(itemKey(place.sitePath, address))));
  view.onToggle((address, expanded) => memory.set(itemKey(place.sitePath, address), rememberedExpanded(expanded)));
}
