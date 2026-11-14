// Port: the folder tree, its checkbox and its button, and where its data comes from.

/** One item of the tree, as `assets/tree.json` holds it. */
export interface TreeItem {
  /** Its words, or their runs, those at odd places feature text that is isolated, the first plain. */
  readonly text: string | readonly string[];
  /** Relative to the site's root, percent-encoded, with an anchor after `#` for a rule or scenario. */
  readonly address: string;
  /** Only a feature's, for its feature text and that of the items inside it. */
  readonly lang?: string;
  readonly items?: readonly TreeItem[];
}

export interface TreeSource {
  /** Null when the data cannot be fetched, or is not a tree. */
  load(): Promise<readonly TreeItem[] | null>;
}

/**
 * The checkbox a wide window shows, the button a narrow one shows in its
 * place, and the landmark they show or hide.
 */
export interface TreeSwitch {
  /** Whether the checkbox is checked as the page's HTML was written, from the configuration. */
  configured(): boolean;
  setChecked(checked: boolean): void;
  /** Says on the button whether the tree is shown. */
  setOpen(open: boolean): void;
  setShown(shown: boolean): void;
  /** The tree cannot be shown: the checkbox and the button are disabled. */
  disable(): void;
  /** The reader checked or unchecked the checkbox. */
  onChange(handler: (checked: boolean) => void): void;
  /** The reader pressed the button. */
  onPress(handler: () => void): void;
  /** The reader pressed Escape inside the tree. */
  onEscape(handler: () => void): void;
  /** Whether the keyboard focus is inside the tree's landmark. */
  holdsFocus(): boolean;
  focusButton(): void;
  focusCheckbox(): void;
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
