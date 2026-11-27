// Port: the blocks of the page, and what the reader does to them.

/** A rule, scenario, background or examples block, the block of the scenarios outside every rule, an examples block's runs, or a folder page's feature, whose contents it shows or hides. */
export interface Block {
  /** The `id` of its heading, or of its item for an examples block, which is its anchor or an `id` of the page's own, or a folder page's feature's link's address. */
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
  /** The block `anchor` names, or the one the element with that `id` is inside; null for neither. */
  holder(anchor: string): Block | null;
  onToggle(handler: (block: Block) => void): void;
  onCollapseAll(handler: () => void): void;
  onExpandAll(handler: () => void): void;
}
