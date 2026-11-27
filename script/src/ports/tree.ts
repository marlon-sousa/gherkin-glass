// Port: the folder tree, its checkbox and its button.

/** One item of the tree. */
export interface TreeItem {
  /** Its words, or their runs, those at odd places feature text that is isolated, the first plain. */
  readonly text: string | readonly string[];
  /** The address of its page, as rule 235 writes it, with its anchor for a rule or a scenario. */
  readonly address: string;
  /** Only a feature's, for its feature text and that of the items inside it. */
  readonly lang?: string;
  readonly items?: readonly TreeItem[];
}

/** Inside the tree's landmark, on the checkbox, on the button, or elsewhere. */
export type FocusPlace = "tree" | "checkbox" | "button" | null;

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
  /** Where the keyboard focus is, of the tree's landmark and its controls. */
  focus(): FocusPlace;
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
