// Service: the folder tree, shown or hidden as the reader chose with the checkbox on a wide window and with the button on a narrow one, with the path to the current page expanded, the checkbox's choice remembered for the whole site.

import { focusHidden, itemKey, rememberedExpanded, rememberedShown, shownKey, startChecked, startExpanded, treeShown } from "../policies/tree_state.ts";
import type { Memory } from "../ports/memory.ts";
import type { TreeItem, TreeSwitch, TreeView } from "../ports/tree.ts";
import type { WindowWidth } from "../ports/width.ts";

/** The tree's controls, which `disable` turns off for good once its items cannot be had. */
export interface TreeControls {
  disable(): void;
}

/** Builds the tree of `items`, the current page's item first in `path`, and every item it is inside after it, expanded. */
export function buildTree(view: TreeView, memory: Memory, sitePath: string, items: readonly TreeItem[], path: readonly string[]): void {
  view.build(items, path[0] ?? null, (address) => startExpanded(path.includes(address), memory.get(itemKey(sitePath, address))));
  view.onToggle((address, expanded) => memory.set(itemKey(sitePath, address), rememberedExpanded(expanded)));
}

/** The checkbox, the button and the width they follow, the site's root at `sitePath`. */
export function startTreeControls(toggle: TreeSwitch, width: WindowWidth, memory: Memory, sitePath: string): TreeControls {
  let checked = startChecked(toggle.configured(), memory.get(shownKey(sitePath)));
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
    memory.set(shownKey(sitePath), rememberedShown(chosen));
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
    const focus = toggle.focus();
    narrow = now;
    open = false;
    show();
    // The focus goes to the control the resize shows in place of what it hid.
    if (focusHidden(focus, narrow, treeShown(narrow, checked, open))) {
      if (narrow) {
        toggle.focusButton();
      } else {
        toggle.focusCheckbox();
      }
    }
  });
  return {
    disable: () => {
      usable = false;
      toggle.disable();
      show();
    },
  };
}
