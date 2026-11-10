// Service: collapsing and expanding the blocks of a page, remembered page by page.

import { memoryKey, remembered, startState } from "../policies/block_state.ts";
import type { Memory } from "../ports/memory.ts";
import type { Block, Page } from "../ports/page.ts";

export function startCollapsing(page: Page, memory: Memory, pagePath: string): void {
  const blocks = page.blocks();
  const set = (block: Block, expanded: boolean): void => {
    block.setExpanded(expanded);
    memory.set(memoryKey(pagePath, block.anchor), remembered(expanded));
  };
  const reveal = (anchor: string): void => {
    let block = blocks.find((found) => found.anchor === anchor) ?? null;
    while (block !== null) {
      if (!block.isExpanded()) {
        set(block, true);
      }
      block = block.parent;
    }
  };

  for (const block of blocks) {
    const kept = memory.get(memoryKey(pagePath, block.anchor));
    block.setExpanded(startState(block.isExpanded(), kept));
  }
  const target = page.target();
  if (target !== null) {
    reveal(target);
  }

  page.onToggle((block) => set(block, !block.isExpanded()));
  page.onCollapseAll(() => blocks.forEach((block) => set(block, false)));
  page.onExpandAll(() => blocks.forEach((block) => set(block, true)));
  page.onFollow(reveal);
}
