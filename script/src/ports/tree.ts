// Port: the folder tree, its checkbox, and where its data comes from.

/** One item of the tree, as `assets/tree.json` holds it. */
export interface TreeItem {
  readonly text: string;
  /** Relative to the site's root, percent-encoded, with an anchor after `#` for a rule or scenario. */
  readonly address: string;
  /** Only a feature's. */
  readonly lang?: string;
  readonly items?: readonly TreeItem[];
}

export interface TreeSource {
  /** Null when the data cannot be fetched, or is not a tree. */
  load(): Promise<readonly TreeItem[] | null>;
}

/** The checkbox, and the landmark it shows or hides. */
export interface TreeSwitch {
  /** As the page's HTML was written, from the configuration. */
  configured(): boolean;
  setShown(shown: boolean): void;
  /** The tree cannot be shown: the checkbox is disabled and the landmark hidden. */
  disable(): void;
  onChange(handler: (shown: boolean) => void): void;
}

export interface TreeView {
  /**
   * Builds the tree in its landmark. `current` is the address of the current
   * page's item, if any; `expanded` says which items start expanded.
   */
  build(items: readonly TreeItem[], current: string | null, expanded: (address: string) => boolean): void;
  /** The reader expanded or collapsed an item. */
  onToggle(handler: (address: string, expanded: boolean) => void): void;
}
