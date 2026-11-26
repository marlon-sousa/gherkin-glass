// Policy: the view of a feature's page, worked out from its object and its descriptor's entry, as docs/BEHAVIOUR.md, "Pages", places its parts.

import { targetAddress } from "./address.ts";
import { type BlockView, type PageContext, type StartingBackground, type TagView, backgroundView, descriptionLines, isScenario, outsideRulesView, ruleView, scenarioView, tagViews } from "./block_views.ts";
import { examplesView } from "./examples_views.ts";
import { type NoteWords, heading, isolated, keywordNote, textOf } from "./heading_words.ts";
import type { Block, Descriptor, FeatureEntry, StoreObject } from "../ports/store.ts";
import type { Parts, Words } from "../ports/words.ts";

export interface ContentsEntry {
  readonly address: string;
  readonly heading: Parts;
  readonly entries: readonly ContentsEntry[];
}

export interface FeatureView {
  readonly title: string;
  readonly lang: string;
  readonly heading: Parts;
  readonly note: NoteWords | null;
  readonly tags: readonly TagView[];
  readonly description: readonly string[];
  /** The link to the file's source, its words with the path isolated. */
  readonly source: { readonly address: string; readonly words: Parts } | null;
  readonly contents: readonly ContentsEntry[];
  readonly controls: {
    readonly collapseAll: string;
    readonly expandAll: string;
    readonly backgrounds: { readonly label: string; readonly checked: boolean; readonly enabled: boolean } | null;
  } | null;
  readonly blocks: readonly BlockView[];
  readonly dialog: { readonly title: string; readonly text: string; readonly close: string } | null;
}

/** The page of `entry`, whose content `object` holds, in the site `descriptor` describes. */
export function featureView(words: Words, descriptor: Descriptor, entry: FeatureEntry, object: StoreObject): FeatureView {
  const feature = object.feature;
  const rules = feature.blocks.filter((block) => block.kind === "rule");
  const scenarios = feature.blocks.filter(isScenario);
  const background = feature.blocks.find((block) => block.kind === "background") ?? null;
  const hasBackground = background !== null || rules.some((rule) => rule.blocks.some((block) => block.kind === "background"));
  const context: PageContext = {
    words,
    start: descriptor.start,
    tags: new Map(descriptor.tags.map((tag) => [tag.tag, tag])),
    hasBackgroundsOption: hasBackground,
  };
  const featureBackground: StartingBackground | null = background && { block: background, heading: words.text("from-background") };
  const startsScenarios = scenarios.length > 0 || rules.some((rule) => rule.blocks.some(isScenario));
  const scenarioLevel = rules.length > 0 && scenarios.length > 0 ? 3 : 2;
  const scenarioViews = scenarios.map((scenario) => scenarioView(context, scenario, scenarioLevel, featureBackground === null ? [] : [featureBackground], (outline, level) => examplesView(context, outline, level)));
  const blocks = [
    ...(background === null ? [] : [backgroundView(context, background, 2, startsScenarios)]),
    ...(scenarioLevel === 3 ? [outsideRulesView(context, scenarioViews)] : scenarioViews),
    ...rules.map((rule) => ruleView(context, rule, featureBackground, (outline, level) => examplesView(context, outline, level))),
  ];
  const featureHeading = heading(words, "feature", feature.name);
  return {
    title: words.text("page-title", { heading: textOf(featureHeading), path: entry.path }),
    lang: object.lang,
    heading: featureHeading,
    note: keywordNote(words, feature),
    tags: tagViews(context, feature.tags),
    description: descriptionLines(feature.description),
    source: entry.source === null ? null : { address: entry.source, words: words.parts("source-link", {}, { path: isolated(entry.path) }) },
    contents: contentsOf(words, entry.page, feature),
    controls:
      blocks.length === 0
        ? null
        : {
            collapseAll: words.text("collapse-all"),
            expandAll: words.text("expand-all"),
            backgrounds: hasBackground ? { label: words.text("inline-backgrounds"), checked: descriptor.start.inline_backgrounds, enabled: startsWithBackground(feature) } : null,
          },
    blocks,
    dialog: hasNote(feature) ? { title: words.text("note-dialog-title"), text: words.text("note-dialog-text"), close: words.text("note-dialog-close") } : null,
  };
}

/** The page's contents, as rule 46 lists them: its rules and scenarios, and the group of those outside every rule. */
export function contentsOf(words: Words, page: string, feature: Block): ContentsEntry[] {
  const link = (anchor: string, parts: Parts, entries: readonly ContentsEntry[] = []): ContentsEntry => ({ address: targetAddress({ kind: "feature", page }, anchor), heading: parts, entries });
  const scenarioEntry = (scenario: Block): ContentsEntry => link(scenario.anchor, heading(words, scenario.kind, scenario.name));
  const rules = feature.blocks.filter((block) => block.kind === "rule");
  const scenarios = feature.blocks.filter(isScenario);
  const ruleEntries = rules.map((rule) => link(rule.anchor, heading(words, "rule", rule.name), rule.blocks.filter(isScenario).map(scenarioEntry)));
  if (rules.length > 0 && scenarios.length > 0) {
    return [link("outside_rules", [{ text: words.text("outside-rules"), isolated: false }], scenarios.map(scenarioEntry)), ...ruleEntries];
  }
  return [...scenarios.map(scenarioEntry), ...ruleEntries];
}

/** Whether a scenario starts with a background's steps, so that the backgrounds' checkbox can act. */
function startsWithBackground(feature: Block): boolean {
  const withSteps = (block: Block): boolean => block.kind === "background" && block.steps.length > 0;
  const featureSteps = feature.blocks.some(withSteps);
  return (featureSteps && feature.blocks.some(isScenario)) || feature.blocks.some((rule) => rule.kind === "rule" && rule.blocks.some(isScenario) && (featureSteps || rule.blocks.some(withSteps)));
}

function hasNote(block: Block): boolean {
  return block.note || block.blocks.some(hasNote) || block.examples.some(hasNote);
}
