// Service: the states of a page's blocks, remembered page by page, with the form its backgrounds show, chosen for the site, which some of those states follow.

import { formKey, rememberedForm, startForm } from "../policies/background_form.ts";
import { formStart, inlineMemoryKey, memoryKey, remembered, startState } from "../policies/block_state.ts";
import type { Backgrounds } from "../ports/backgrounds.ts";
import type { Memory } from "../ports/memory.ts";
import type { Block, Page } from "../ports/page.ts";

/** The checkbox of a page with a background, and the site whose pages share the reader's choice. */
export interface BackgroundsOption {
  readonly backgrounds: Backgrounds;
  readonly siteRoot: string;
}

export function startCollapsing(page: Page, memory: Memory, pagePath: string, option: BackgroundsOption | null): void {
  const blocks = page.blocks();
  const followingTheForm = blocks.filter((block) => block.expandedWhenLinked !== null);
  let inline = false;
  const key = (block: Block): string =>
    inline && block.expandedWhenLinked !== null ? inlineMemoryKey(pagePath, block.anchor) : memoryKey(pagePath, block.anchor);
  const set = (block: Block, expanded: boolean): void => {
    block.setExpanded(expanded);
    memory.set(key(block), remembered(expanded));
  };
  // A block's state as the page was written is its state until it is first settled, and a background that follows the form ignores it.
  const settle = (block: Block): void => {
    const start = formStart(block.isExpanded(), block.expandedWhenLinked, inline);
    block.setExpanded(startState(start, memory.get(key(block))));
  };
  // Another page of the site may have changed the choice while this one was left; with nothing remembered, `unchosen` stands.
  const showForm = (unchosen: boolean): void => {
    if (option !== null) {
      inline = startForm(unchosen, memory.get(formKey(option.siteRoot)));
      option.backgrounds.setInline(inline);
    }
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

  showForm(option?.backgrounds.writtenInline() ?? false);
  blocks.forEach(settle);
  const target = page.target();
  if (target !== null) {
    reveal(target);
  }

  page.onToggle((block) => set(block, !block.isExpanded()));
  page.onCollapseAll(() => blocks.forEach((block) => set(block, false)));
  page.onExpandAll(() => blocks.forEach((block) => set(block, true)));
  page.onFollow(reveal);
  if (option !== null) {
    option.backgrounds.onSwitch((switched) => {
      inline = switched;
      option.backgrounds.setInline(inline);
      memory.set(formKey(option.siteRoot), rememberedForm(inline));
      followingTheForm.forEach((block) => set(block, !inline));
    });
    option.backgrounds.onReturn(() => {
      const shown = inline;
      showForm(shown);
      if (inline !== shown) {
        followingTheForm.forEach(settle);
      }
    });
  }
}
