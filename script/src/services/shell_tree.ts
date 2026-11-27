// Service: the folder tree of a shell, built from the descriptor and the objects of the store, with the current page's item marked each time a page is drawn.

import { startTreeControls, buildTree } from "./folder_tree.ts";
import { loadedFeatures } from "./loaded_features.ts";
import type { PageName } from "../policies/address.ts";
import { pathToPage, treeItems } from "../policies/tree_items.ts";
import type { Memory } from "../ports/memory.ts";
import type { Store } from "../ports/store.ts";
import type { TreeItem, TreeSwitch, TreeView } from "../ports/tree.ts";
import type { WindowWidth } from "../ports/width.ts";
import type { Words } from "../ports/words.ts";

export interface ShellTreeParts {
  readonly store: Store;
  readonly words: Words;
  readonly toggle: TreeSwitch;
  readonly view: TreeView;
  readonly width: WindowWidth;
  readonly memory: Memory;
  /** The path of the shell's folder, ending in `/`, under which the reader's choices are kept. */
  readonly sitePath: string;
}

export interface ShellTree {
  /** Marks the item of `page`, the page just drawn, once the tree's items are had. */
  show(page: PageName): void;
  /** Leaves the tree empty and hidden, its controls disabled. */
  disable(): void;
}

export function startShellTree(parts: ShellTreeParts): ShellTree {
  const controls = startTreeControls(parts.toggle, parts.width, parts.memory, parts.sitePath);
  let items: readonly TreeItem[] | null = null;
  let current: PageName | null = null;
  let usable = true;
  const build = (): void => {
    if (usable && items !== null && current !== null) {
      buildTree(parts.view, parts.memory, parts.sitePath, items, pathToPage(items, current));
    }
  };
  const disable = (): void => {
    if (usable) {
      usable = false;
      controls.disable();
    }
  };
  void (async () => {
    try {
      const descriptor = await parts.store.descriptor();
      items = treeItems(parts.words, descriptor, await loadedFeatures(parts.store, descriptor));
      build();
    } catch {
      disable();
    }
  })();
  return {
    show: (page) => {
      current = page;
      build();
    },
    disable,
  };
}
