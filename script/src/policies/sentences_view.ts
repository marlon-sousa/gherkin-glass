// Policy: the view of the page of step sentences, the descriptor's sentences with every step of its features' objects that uses each.

import { blockHeading, type Piece, pieces } from "./heading_words.ts";
import type { LoadedFeature } from "./site_folders.ts";
import { targetAddress } from "./address.ts";
import type { Block, Descriptor, StepKind } from "../ports/store.ts";
import type { Parts, Words } from "../ports/words.ts";

export interface SentenceUseView {
  readonly address: string;
  /** The link's words, with the slots `heading` and `path`. */
  readonly link: readonly Piece[];
  /** The item's words, with the slots `link` and `values`, or null for a step with no values. */
  readonly item: readonly Piece[] | null;
  readonly heading: Parts;
  readonly path: string;
  readonly values: readonly string[];
  readonly separator: string;
  /** The `lang` of its file's text, null when it is `en`. */
  readonly lang: string | null;
}

export interface SentenceView {
  readonly text: string;
  /** The `lang` of the file of its first use, null when it is `en`. */
  readonly lang: string | null;
  readonly count: string;
  readonly uses: readonly SentenceUseView[];
}

export interface SentencesView {
  readonly title: string;
  readonly heading: string;
  readonly groups: readonly { readonly heading: string; readonly sentences: readonly SentenceView[] }[];
  /** Said in place of the groups when no feature has a step. */
  readonly none: string | null;
}

const KINDS: readonly StepKind[] = ["given", "when", "then"];

export function sentencesView(words: Words, descriptor: Descriptor, features: readonly LoadedFeature[]): SentencesView {
  const uses = usesOf(words, features);
  const groups = KINDS.filter((kind) => descriptor.sentences[kind].length > 0).map((kind) => ({
    heading: words.text(`sentences-${kind}`),
    sentences: descriptor.sentences[kind].map((text) => {
      const found = uses.get(useKey(kind, text)) ?? [];
      return { text, lang: found[0]?.lang ?? null, count: words.text("sentence-uses", { count: found.length }), uses: found };
    }),
  }));
  return {
    title: words.text("sentences-page-title", { site: descriptor.title }),
    heading: words.text("sentences-title"),
    groups,
    none: groups.length === 0 ? words.text("sentences-none") : null,
  };
}

/** Every step's use, by its kind and its sentence, in the order of the features and of each file. */
function usesOf(words: Words, features: readonly LoadedFeature[]): Map<string, SentenceUseView[]> {
  const uses = new Map<string, SentenceUseView[]>();
  const link = pieces(words, "sentence-use", {}, ["heading", "path"]);
  const separator = words.text("sentence-values-separator");
  for (const { entry, object } of features) {
    const lang = object.lang === "en" ? null : object.lang;
    for (const block of blocksWithSteps(object.feature)) {
      for (const step of block.steps) {
        const use: SentenceUseView = {
          address: targetAddress({ kind: "feature", page: entry.page }, block.anchor),
          link,
          item: step.slots.length === 0 ? null : pieces(words, "sentence-use-values", {}, ["link", "values"]),
          heading: blockHeading(words, block),
          path: entry.path,
          values: step.slots,
          separator,
          lang,
        };
        const key = useKey(step.kind, step.sentence);
        const known = uses.get(key);
        if (known === undefined) {
          uses.set(key, [use]);
        } else {
          known.push(use);
        }
      }
    }
  }
  return uses;
}

/** The blocks of `block` that have steps of their own, in the order of the file. */
function blocksWithSteps(block: Block): Block[] {
  return block.blocks.flatMap((inner) => [...(inner.steps.length > 0 ? [inner] : []), ...blocksWithSteps(inner)]);
}

function useKey(kind: StepKind, sentence: string): string {
  // A sentence holds no line feed, a step being one line.
  return `${kind}\n${sentence}`;
}
