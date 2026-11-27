// Adapter: a feature's page drawn from its view, in the elements and classes feature_page.html, feature_block.html, examples_list.html and examples_item.html in src/policies/rendering write.

import { descriptionOf, noteOf, stepList, tableOf, tagList } from "./dom_feature_parts.ts";
import type { ElementBuilder } from "./element_builder.ts";
import type { BlockView, ExamplesItemView, ExamplesView } from "../policies/block_views.ts";
import type { ContentsEntry, FeatureView } from "../policies/feature_view.ts";

/** Draws, inside each block's content and first in it, what `firstInBlock` makes for the block, such as its lists in other versions. */
export type BlockAddition = (block: BlockView) => Node | null;

/** The content of a feature's `main`; `contentsName` is the contents landmark's. */
export function featurePage(make: ElementBuilder, view: FeatureView, contentsName: string, firstInBlock: BlockAddition, afterHeading: Node | null): Node[] {
  const controls = view.controls;
  return make.nodes([
    make.make("h1", {}, ...make.words(view.heading)),
    afterHeading,
    noteOf(make, view.note),
    tagList(make, view.tags),
    descriptionOf(make, view.description),
    view.source && make.make("p", { class: "ghg-source" }, make.make("a", { href: view.source.address }, ...make.words(view.source.words))),
    view.contents.length > 0 && make.make("nav", { class: "ghg-contents", "aria-label": contentsName }, contentsList(make, view.contents, null)),
    controls &&
      make.make(
        "div",
        { class: "ghg-controls" },
        make.make("button", { type: "button", class: "ghg-collapse-all" }, controls.collapseAll),
        make.make("button", { type: "button", class: "ghg-expand-all" }, controls.expandAll),
        controls.backgrounds &&
          make.make(
            "label",
            { class: "ghg-option" },
            make.make("input", { type: "checkbox", class: "ghg-inline-backgrounds", autocomplete: "off", checked: controls.backgrounds.checked, disabled: !controls.backgrounds.enabled }),
            ` ${controls.backgrounds.label}`,
          ),
      ),
    ...view.blocks.map((block) => blockOf(make, block, firstInBlock)),
    view.dialog &&
      make.make(
        "dialog",
        { id: "ghg_note_dialog", class: "ghg-dialog", tabindex: "-1", "aria-labelledby": "ghg_note_dialog_title", "aria-describedby": "ghg_note_dialog_text" },
        make.make("h2", { id: "ghg_note_dialog_title", class: "ghg-dialog-title" }, view.dialog.title),
        make.make("p", { id: "ghg_note_dialog_text" }, view.dialog.text),
        make.make("form", { method: "dialog" }, make.make("button", {}, view.dialog.close)),
      ),
  ]);
}

/** A list of links to a page's blocks, each name in `lang` when it is given, as a folder page shows them. */
export function contentsList(make: ElementBuilder, entries: readonly ContentsEntry[], lang: string | null): HTMLUListElement {
  return make.make(
    "ul",
    {},
    ...entries.map((entry) => make.make("li", {}, make.make("a", { href: entry.address }, ...make.words(entry.heading, lang)), entry.entries.length > 0 && contentsList(make, entry.entries, lang))),
  );
}

function blockOf(make: ElementBuilder, block: BlockView, firstInBlock: BlockAddition): HTMLElement {
  const heading = make.make(headingTag(block.level), { id: block.id }, toggle(make, block.expanded, make.words(block.heading)));
  const content = make.make(
    "div",
    { class: "ghg-content", hidden: !block.expanded },
    firstInBlock(block),
    noteOf(make, block.note),
    tagList(make, block.tags),
    descriptionOf(make, block.description),
    ...block.backgrounds.map((background) => blockOf(make, background, firstInBlock)),
    block.stepsHeading && make.make(headingTag(block.stepsHeading.level), { class: "ghg-steps-heading" }, block.stepsHeading.text),
    stepList(make, block.steps),
    block.examples && examplesList(make, block.examples, firstInBlock),
    ...block.blocks.map((inner) => blockOf(make, inner, firstInBlock)),
  );
  const unchecked = block.expandedWhenUnchecked === null ? null : String(block.expandedWhenUnchecked);
  return make.make("div", { class: `ghg-block ${block.className}`, "data-expanded-when-unchecked": unchecked }, heading, content);
}

function examplesList(make: ElementBuilder, examples: ExamplesView, firstInBlock: BlockAddition): Node {
  return make.fragment(
    examples.runsParagraph && make.make("p", { id: examples.runsParagraph.id, class: "ghg-runs" }, examples.runsParagraph.text),
    make.make("ul", { class: "ghg-examples-list" }, ...examples.items.map((item) => examplesItem(make, item, firstInBlock))),
  );
}

function examplesItem(make: ElementBuilder, item: ExamplesItemView, firstInBlock: BlockAddition): HTMLElement {
  if (item.holdsNothing) {
    return make.make("li", { id: item.id, class: "ghg-examples" }, make.make("button", { type: "button", class: "ghg-toggle", "aria-disabled": "true" }, ...make.words(item.label)));
  }
  const content = make.make(
    "div",
    { class: "ghg-content", hidden: true },
    noteOf(make, item.note),
    tagList(make, item.tags),
    descriptionOf(make, item.description),
    item.table && tableOf(make, item.table),
    item.runs && blockOf(make, item.runs, firstInBlock),
  );
  return make.make("li", { id: item.id, class: "ghg-block ghg-examples" }, toggle(make, false, make.words(item.label)), content);
}

function toggle(make: ElementBuilder, expanded: boolean, words: readonly Node[]): HTMLButtonElement {
  return make.make("button", { type: "button", class: "ghg-toggle", "aria-expanded": String(expanded) }, ...words);
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
