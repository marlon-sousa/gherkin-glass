// Policy: the view of an outline's list of examples, each item with its table and the scenarios its rows produce, as docs/BEHAVIOUR.md, rules 45 and 129 to 132, places them.

import { type BlockView, EMPTY_BLOCK, type ExamplesItemView, type ExamplesView, type PageContext, descriptionLines, outlineName, stepViews, tableView, tagViews } from "./block_views.ts";
import { isolated, keywordNote, blockHeading, plain } from "./heading_words.ts";
import type { Block, Run } from "../ports/store.ts";
import type { Parts } from "../ports/words.ts";

/** The list of `outline`'s examples, its runs' headings at `level`; null for a block with no examples. */
export function examplesView(context: PageContext, outline: Block, level: number): ExamplesView | null {
  if (outline.examples.length === 0) {
    return null;
  }
  const hasRows = outline.examples.some((examples) => examples.runs.length > 0);
  return {
    runsParagraph: hasRows ? { id: `${outline.anchor}_scenarios`, text: context.words.text("runs-for-each-row") } : null,
    items: outline.examples.map((examples) => examplesItem(context, outline, examples, level)),
  };
}

function examplesItem(context: PageContext, outline: Block, examples: Block, level: number): ExamplesItemView {
  const heading = blockHeading(context.words, examples);
  const note = keywordNote(context.words, examples);
  const table = tableView(examples.table);
  const description = descriptionLines(examples.description);
  return {
    id: examples.anchor,
    label: examples.runs.length > 0 ? heading : context.words.parts("nothing-to-run", {}, { heading }),
    holdsNothing: note === null && table === null && examples.tags.length === 0 && description.length === 0,
    note,
    tags: tagViews(context, examples.tags),
    description,
    table,
    runs: examples.runs.length === 0 ? null : runsView(context, outline, examples, level),
  };
}

function runsView(context: PageContext, outline: Block, examples: Block, level: number): BlockView {
  const [columns = [], ...rows] = examples.table ?? [];
  return {
    ...EMPTY_BLOCK,
    className: "ghg-examples-as-scenarios",
    level,
    id: `${examples.anchor}_runs`,
    heading: plain(context.words.text("runs-one-per-row")),
    expanded: false,
    blocks: examples.runs.map((run, row) => runView(context, outline, run, columns, rows[row] ?? [], level + 1)),
  };
}

function runView(context: PageContext, outline: Block, run: Run, columns: readonly string[], cells: readonly string[], level: number): BlockView {
  return {
    ...EMPTY_BLOCK,
    className: "ghg-scenario",
    level,
    id: run.anchor,
    heading: context.words.parts("example-scenario-name", {}, { name: outlineName(context.words, outline), values: rowValues(context, columns, cells) }),
    expanded: !context.start.collapse_scenarios,
    steps: stepViews(run),
  };
}

/** Each column's name and the row's value, in the order of the table. */
function rowValues(context: PageContext, columns: readonly string[], cells: readonly string[]): Parts {
  const separator = plain(context.words.text("example-values-separator"));
  return columns.flatMap((column, place) => [
    ...(place === 0 ? [] : separator),
    ...context.words.parts("example-value", {}, { column: isolated(column), value: isolated(cells[place] ?? "") }),
  ]);
}
