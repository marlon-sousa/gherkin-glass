// Policy: the view of each block of a feature's page, its heading's level and words, the state it starts in, its steps and the backgrounds a scenario starts with.

import { pageByName, targetAddress } from "./address.ts";
import { type NoteWords, blockHeading, heading, keywordNote, plain } from "./heading_words.ts";
import type { Block, DocString, Part, StartStates, Table, TagEntry } from "../ports/store.ts";
import type { Parts, Words } from "../ports/words.ts";

export type BlockClass = "ghg-background" | "ghg-background-in-scenario" | "ghg-rule" | "ghg-outside-rules" | "ghg-scenario" | "ghg-examples-as-scenarios";

export interface TagView {
  readonly text: string;
  readonly address: string;
  readonly link: { readonly address: string; readonly text: string } | null;
}

export interface TableView {
  readonly header: readonly string[];
  readonly rows: Table;
}

export interface StepView {
  /** As the language writes it, with the space after it when it has one. */
  readonly keyword: string;
  readonly parts: readonly Part[];
  readonly table: TableView | null;
  readonly docString: DocString | null;
}

export interface ExamplesItemView {
  readonly id: string;
  readonly label: Parts;
  /** An item with nothing to show holds a button that opens nothing. */
  readonly holdsNothing: boolean;
  readonly note: NoteWords | null;
  readonly tags: readonly TagView[];
  readonly description: readonly string[];
  readonly table: TableView | null;
  readonly runs: BlockView | null;
}

export interface ExamplesView {
  /** The paragraph before the list, with its `id`, when a block has a row. */
  readonly runsParagraph: { readonly id: string; readonly text: string } | null;
  readonly items: readonly ExamplesItemView[];
}

export interface BlockView {
  readonly className: BlockClass;
  readonly level: number;
  readonly id: string;
  readonly heading: Parts;
  readonly expanded: boolean;
  /** For a background the backgrounds' checkbox sets, whether it is expanded while the box is unchecked. */
  readonly expandedWhenUnchecked: boolean | null;
  /** The block's own anchor, which its lists in other versions name; null for a block of the page's own. */
  readonly anchor: string | null;
  readonly note: NoteWords | null;
  readonly tags: readonly TagView[];
  readonly description: readonly string[];
  readonly backgrounds: readonly BlockView[];
  readonly stepsHeading: { readonly level: number; readonly text: string } | null;
  readonly steps: readonly StepView[];
  readonly examples: ExamplesView | null;
  readonly blocks: readonly BlockView[];
}

/** What every block of one page is made from. */
export interface PageContext {
  readonly words: Words;
  readonly start: StartStates;
  readonly tags: ReadonlyMap<string, TagEntry>;
  /** Whether the page has the backgrounds' checkbox, which heads a scenario's own steps after its backgrounds. */
  readonly hasBackgroundsOption: boolean;
}

/** A background a scenario starts with: its steps, and the words of its heading there. */
export interface StartingBackground {
  readonly block: Block;
  readonly heading: string;
}

const EMPTY: Omit<BlockView, "className" | "level" | "id" | "heading" | "expanded"> = {
  expandedWhenUnchecked: null,
  anchor: null,
  note: null,
  tags: [],
  description: [],
  backgrounds: [],
  stepsHeading: null,
  steps: [],
  examples: null,
  blocks: [],
};

export function tagViews(context: PageContext, tags: readonly string[]): TagView[] {
  return tags.map((tag) => {
    const entry = context.tags.get(tag);
    return { text: tag, address: targetAddress(pageByName(entry?.page ?? "tags/")), link: entry?.link ?? null };
  });
}

export function descriptionLines(description: string): readonly string[] {
  return description === "" ? [] : description.split("\n");
}

export function tableView(table: Table | null): TableView | null {
  const [header, ...rows] = table ?? [];
  return header === undefined ? null : { header, rows };
}

export function stepViews(block: { readonly steps: Block["steps"] }): StepView[] {
  return block.steps.map((step) => ({ keyword: step.keyword, parts: step.parts, table: tableView(step.table), docString: step.doc_string }));
}

/** A background in its own place, at `level`; `startsScenarios` says whether a scenario below it starts with its steps. */
export function backgroundView(context: PageContext, background: Block, level: number, startsScenarios: boolean): BlockView {
  const follows = startsScenarios && background.steps.length > 0;
  const collapsed = (follows && context.start.inline_backgrounds) || context.start.collapse_scenarios;
  return {
    ...EMPTY,
    className: "ghg-background",
    level,
    id: background.anchor,
    anchor: background.anchor,
    heading: blockHeading(context.words, background),
    expanded: !collapsed,
    expandedWhenUnchecked: follows ? !context.start.collapse_scenarios : null,
    note: keywordNote(context.words, background),
    description: descriptionLines(background.description),
    steps: stepViews(background),
  };
}

/** A scenario or an outline at `level`, starting with the steps of `backgrounds`. */
export function scenarioView(context: PageContext, scenario: Block, level: number, backgrounds: readonly StartingBackground[], examples: (outline: Block, level: number) => ExamplesView | null): BlockView {
  const inScenario = backgrounds
    .filter((background) => background.block.steps.length > 0)
    .map((background, place) => backgroundInScenario(context, background, `${scenario.anchor}_background_${place + 1}`, level + 1));
  const steps = stepViews(scenario);
  return {
    ...EMPTY,
    className: "ghg-scenario",
    level,
    id: scenario.anchor,
    anchor: scenario.anchor,
    heading: blockHeading(context.words, scenario),
    expanded: !context.start.collapse_scenarios,
    note: keywordNote(context.words, scenario),
    tags: tagViews(context, scenario.tags),
    description: descriptionLines(scenario.description),
    backgrounds: inScenario,
    stepsHeading: context.hasBackgroundsOption && inScenario.length > 0 && steps.length > 0 ? { level: level + 1, text: context.words.text("from-scenario") } : null,
    steps,
    examples: examples(scenario, level + 1),
  };
}

function backgroundInScenario(context: PageContext, background: StartingBackground, id: string, level: number): BlockView {
  return {
    ...EMPTY,
    className: "ghg-background-in-scenario",
    level,
    id,
    heading: plain(background.heading),
    expanded: context.start.inline_backgrounds,
    expandedWhenUnchecked: false,
    steps: stepViews(background.block),
  };
}

/** A rule, its background and its scenarios, starting each with `featureBackground` and its own. */
export function ruleView(context: PageContext, rule: Block, featureBackground: StartingBackground | null, examples: (outline: Block, level: number) => ExamplesView | null): BlockView {
  const scenarios = rule.blocks.filter(isScenario);
  const background = rule.blocks.find((block) => block.kind === "background") ?? null;
  const starting = [featureBackground, background && { block: background, heading: context.words.text("from-rule-background") }].filter((found) => found !== null);
  return {
    ...EMPTY,
    className: "ghg-rule",
    level: 2,
    id: rule.anchor,
    anchor: rule.anchor,
    heading: blockHeading(context.words, rule),
    expanded: !context.start.collapse_rules,
    note: keywordNote(context.words, rule),
    tags: tagViews(context, rule.tags),
    description: descriptionLines(rule.description),
    blocks: [
      ...(background === null ? [] : [backgroundView(context, background, 3, scenarios.length > 0)]),
      ...scenarios.map((scenario) => scenarioView(context, scenario, 3, starting, examples)),
    ],
  };
}

/** The block of the scenarios outside every rule, in a file with a rule. */
export function outsideRulesView(context: PageContext, scenarios: readonly BlockView[]): BlockView {
  return { ...EMPTY, className: "ghg-outside-rules", level: 2, id: "outside_rules", heading: plain(context.words.text("outside-rules")), expanded: !context.start.collapse_rules, blocks: scenarios };
}

export function isScenario(block: Block): boolean {
  return block.kind === "scenario" || block.kind === "scenario-outline";
}

/** A produced scenario's heading reads its outline's kind when the outline has no name. */
export function outlineName(words: Words, outline: Block): Parts {
  return outline.name === "" ? heading(words, "scenario-outline", "") : [{ text: outline.name, isolated: true }];
}

export { EMPTY as EMPTY_BLOCK };
