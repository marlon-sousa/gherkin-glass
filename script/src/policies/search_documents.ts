// Policy: the documents search reads, made from a descriptor's features and their objects, as docs/BEHAVIOUR.md, rules 147 and 148, list them.

import type { LoadedFeature } from "./site_folders.ts";
import type { SearchDocument } from "../ports/documents.ts";
import type { Block } from "../ports/store.ts";

/** One entry for each feature, rule, background, scenario and examples block, in the order of the features and of each file. */
export function searchDocuments(features: readonly LoadedFeature[]): SearchDocument[] {
  return features.flatMap(({ entry, object }) => {
    const feature = object.feature;
    const document = (block: Block, rule: Block | null, outline: Block | null, tags: readonly string[]): SearchDocument => ({
      kind: block.kind,
      name: block.name,
      feature: block === feature ? null : feature.name,
      rule: rule === null ? null : rule.name,
      outline: outline === null ? null : outline.name,
      place: outline === null ? null : block.place,
      steps: block.steps.map((step) => step.text),
      tags,
      cells: block.table?.flat() ?? [],
      page: entry.page,
      anchor: block === feature ? "" : block.anchor,
    });
    const inside = (blocks: readonly Block[], rule: Block | null): SearchDocument[] =>
      blocks.flatMap((block) =>
        block.kind === "rule"
          ? [document(block, null, null, block.tags), ...inside(block.blocks, block)]
          : [document(block, rule, null, outlineTags(block)), ...block.examples.map((examples) => document(examples, rule, block, examples.tags))],
      );
    return [document(feature, null, null, feature.tags), ...inside(feature.blocks, null)];
  });
}

/** A block's own tags, and an outline's those of its examples blocks after them, each once. */
function outlineTags(block: Block): string[] {
  return [...new Set([...block.tags, ...block.examples.flatMap((examples) => examples.tags)])];
}
