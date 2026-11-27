// Adapter: the markup of a feature's page, drawn from its view, in the elements and classes the stylesheet styles.

import { descriptionOf, noteOf, stepList, tableOf, tagList } from "./dom_feature_parts.tsx";
import type { Markup } from "./jsx/jsx_runtime.ts";
import { Words } from "./jsx/words.tsx";
import type { BlockView, ExamplesItemView, ExamplesView } from "../policies/block_views.ts";
import type { ContentsEntry, FeatureView } from "../policies/feature_view.ts";
import type { Parts } from "../ports/words.ts";

/** Draws, inside each block's content and first in it, what `firstInBlock` makes for the block, such as its lists in other versions. */
export type BlockAddition = (block: BlockView) => Markup | null;

/** The content of a feature's `main`; `contentsName` is the contents landmark's. */
export function featurePage(view: FeatureView, contentsName: string, firstInBlock: BlockAddition, afterHeading: Markup | null): Markup {
  const controls = view.controls;
  return (
    <>
      <h1>
        <Words parts={view.heading} />
      </h1>
      {afterHeading}
      {noteOf(view.note)}
      {tagList(view.tags)}
      {descriptionOf(view.description)}
      {view.source && (
        <p class="ghg-source">
          <a href={view.source.address}>
            <Words parts={view.source.words} />
          </a>
        </p>
      )}
      {view.contents.length > 0 && (
        <nav class="ghg-contents" aria-label={contentsName}>
          {contentsList(view.contents, null)}
        </nav>
      )}
      {controls && (
        <div class="ghg-controls">
          <button type="button" class="ghg-collapse-all">
            {controls.collapseAll}
          </button>
          <button type="button" class="ghg-expand-all">
            {controls.expandAll}
          </button>
          {controls.backgrounds && (
            <label class="ghg-option">
              <input type="checkbox" class="ghg-inline-backgrounds" autocomplete="off" checked={controls.backgrounds.checked} disabled={!controls.backgrounds.enabled} />
              {` ${controls.backgrounds.label}`}
            </label>
          )}
        </div>
      )}
      {view.blocks.map((block) => blockOf(block, firstInBlock))}
      {view.dialog && (
        <dialog id="ghg_note_dialog" class="ghg-dialog" tabindex="-1" aria-labelledby="ghg_note_dialog_title" aria-describedby="ghg_note_dialog_text">
          <h2 id="ghg_note_dialog_title" class="ghg-dialog-title">
            {view.dialog.title}
          </h2>
          <p id="ghg_note_dialog_text">{view.dialog.text}</p>
          <form method="dialog">
            <button>{view.dialog.close}</button>
          </form>
        </dialog>
      )}
    </>
  );
}

/** A list of links to a page's blocks, each name in `lang` when it is given, as a folder page shows them. */
export function contentsList(entries: readonly ContentsEntry[], lang: string | null): Markup {
  return (
    <ul>
      {entries.map((entry) => (
        <li>
          <a href={entry.address}>
            <Words parts={entry.heading} lang={lang} />
          </a>
          {entry.entries.length > 0 && contentsList(entry.entries, lang)}
        </li>
      ))}
    </ul>
  );
}

function blockOf(block: BlockView, firstInBlock: BlockAddition): Markup {
  const Heading = headingTag(block.level);
  const unchecked = block.expandedWhenUnchecked === null ? null : String(block.expandedWhenUnchecked);
  return (
    <div class={`ghg-block ${block.className}`} data-expanded-when-unchecked={unchecked}>
      <Heading id={block.id}>{toggle(block.expanded, block.heading)}</Heading>
      <div class="ghg-content" hidden={!block.expanded}>
        {firstInBlock(block)}
        {noteOf(block.note)}
        {tagList(block.tags)}
        {descriptionOf(block.description)}
        {block.backgrounds.map((background) => blockOf(background, firstInBlock))}
        {block.stepsHeading && stepsHeadingOf(block.stepsHeading)}
        {stepList(block.steps)}
        {block.examples && examplesList(block.examples, firstInBlock)}
        {block.blocks.map((inner) => blockOf(inner, firstInBlock))}
      </div>
    </div>
  );
}

function examplesList(examples: ExamplesView, firstInBlock: BlockAddition): Markup {
  return (
    <>
      {examples.runsParagraph && (
        <p id={examples.runsParagraph.id} class="ghg-runs">
          {examples.runsParagraph.text}
        </p>
      )}
      <ul class="ghg-examples-list">{examples.items.map((item) => examplesItem(item, firstInBlock))}</ul>
    </>
  );
}

function examplesItem(item: ExamplesItemView, firstInBlock: BlockAddition): Markup {
  if (item.holdsNothing) {
    return (
      <li id={item.id} class="ghg-examples">
        <button type="button" class="ghg-toggle" aria-disabled="true">
          <Words parts={item.label} />
        </button>
      </li>
    );
  }
  return (
    <li id={item.id} class="ghg-block ghg-examples">
      {toggle(false, item.label)}
      <div class="ghg-content" hidden>
        {noteOf(item.note)}
        {tagList(item.tags)}
        {descriptionOf(item.description)}
        {item.table && tableOf(item.table)}
        {item.runs && blockOf(item.runs, firstInBlock)}
      </div>
    </li>
  );
}

function stepsHeadingOf(heading: NonNullable<BlockView["stepsHeading"]>): Markup {
  const Heading = headingTag(heading.level);
  return <Heading class="ghg-steps-heading">{heading.text}</Heading>;
}

function toggle(expanded: boolean, words: Parts): Markup {
  return (
    <button type="button" class="ghg-toggle" aria-expanded={expanded ? "true" : "false"}>
      <Words parts={words} />
    </button>
  );
}

function headingTag(level: number): "h2" | "h3" | "h4" | "h5" | "h6" {
  switch (level) {
    case 2:
      return "h2";
    case 3:
      return "h3";
    case 4:
      return "h4";
    case 5:
      return "h5";
    default:
      return "h6";
  }
}
