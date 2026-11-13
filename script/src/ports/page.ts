// Port: the blocks of the page, and what the reader does to them.

/** A rule, scenario, background or examples block, an outline's examples as scenarios, or a folder page's feature, whose contents it shows or hides. */
export interface Block {
  /** The `id` of its heading, which is its anchor or an `id` of the page's own, or a folder page's feature's link's address. */
  readonly anchor: string;
  /** The block it is inside, if any. */
  readonly parent: Block | null;
  /** For a background whose state the backgrounds' checkbox sets, whether it starts expanded while the box is unchecked; null for any other block. */
  readonly expandedWhenUnchecked: boolean | null;
  /** Whether it is a background inside a scenario, which the checked box expands, rather than one in its own place, which the checked box collapses. */
  readonly inScenario: boolean;
  isExpanded(): boolean;
  setExpanded(expanded: boolean): void;
}

export interface Page {
  /** In the order of the page. */
  blocks(): readonly Block[];
  /** The anchor the page was opened at, if any. */
  target(): string | null;
  onToggle(handler: (block: Block) => void): void;
  onCollapseAll(handler: () => void): void;
  onExpandAll(handler: () => void): void;
  /** A link on the page to one of its anchors was followed. */
  onFollow(handler: (anchor: string) => void): void;
}
