// Policy: the views of the index of tags and of a tag's page, worked out from the descriptor and its features' objects.

import { pageByName, targetAddress } from "./address.ts";
import { isScenario } from "./block_views.ts";
import { heading, isolated, wordsAround } from "./heading_words.ts";
import type { LoadedFeature } from "./site_folders.ts";
import type { Block, Descriptor, TagEntry, TagLink } from "../ports/store.ts";
import type { Parts, Words } from "../ports/words.ts";

/** A link drawn between words: those before it, its own, and those after it. */
export interface LinkInWords {
  readonly before: Parts;
  readonly address: string;
  readonly words: Parts;
  readonly after: Parts;
}

export interface TagIndexView {
  readonly title: string;
  /** One link to each tag's page, the tag isolated; null when no tag is written. */
  readonly entries: readonly LinkInWords[] | null;
  readonly empty: string;
}

export interface TagFeatureView {
  readonly address: string;
  /** The words of the feature's page's title before its path, and after it. */
  readonly before: Parts;
  readonly path: string;
  readonly after: Parts;
  /** The `lang` of its feature text, null when it is `en`. */
  readonly lang: string | null;
  /** The scenarios that carry the tag, null when none does. */
  readonly scenarios: readonly LinkInWords[] | null;
  readonly none: string;
}

export interface TagPageView {
  readonly title: string;
  readonly heading: Parts;
  readonly outside: TagLink | null;
  readonly features: readonly TagFeatureView[];
}

/** What gives a scenario its tag, the first of these that carries it. */
type Carrier = "itself" | "examples" | "rule" | "feature";

interface Carried {
  readonly scenario: Block;
  readonly carrier: Carrier;
}

export function tagIndexView(words: Words, descriptor: Descriptor, features: readonly LoadedFeature[]): TagIndexView {
  return {
    title: words.text("tag-index-title"),
    entries:
      descriptor.tags.length === 0
        ? null
        : descriptor.tags.map((entry) => {
            const count = features.reduce((total, feature) => total + carriedIn(feature.object.feature, entry.tag).length, 0);
            return { ...wordsAround(words, "tag-index-entry", { count }, {}, "tag"), address: tagAddress(entry), words: isolated(entry.tag) };
          }),
    empty: words.text("tag-index-empty"),
  };
}

/** The page of the tag whose page's name is `tags/<name>`, or null when no tag has it. */
export function tagPageView(words: Words, descriptor: Descriptor, features: readonly LoadedFeature[], name: string): TagPageView | null {
  const entry = descriptor.tags.find((candidate) => candidate.page === `tags/${name}`);
  if (entry === undefined) {
    return null;
  }
  return {
    title: words.text("tag-page-title", { tag: entry.tag }),
    heading: words.parts("tag-page-title", {}, { tag: isolated(entry.tag) }),
    outside: entry.link,
    features: features.filter((feature) => writtenIn(feature.object.feature, entry.tag)).map((feature) => tagFeature(words, feature, entry.tag)),
  };
}

function tagFeature(words: Words, { entry, object }: LoadedFeature, tag: string): TagFeatureView {
  const page = { kind: "feature", page: entry.page } as const;
  const carried = carriedIn(object.feature, tag);
  return {
    address: targetAddress(page),
    ...wordsAround(words, "page-title", {}, { heading: heading(words, "feature", object.feature.name) }, "path"),
    path: entry.path,
    lang: object.lang === "en" ? null : object.lang,
    scenarios:
      carried.length === 0
        ? null
        : carried.map(({ scenario, carrier }) => {
            const link = { address: targetAddress(page, scenario.anchor), words: heading(words, scenario.kind, scenario.name) };
            return carrier === "itself" ? { ...link, before: [], after: [] } : { ...link, ...wordsAround(words, `tag-through-${carrier}`, {}, {}, "scenario") };
          }),
    none: words.text("tag-no-scenario"),
  };
}

/** The feature's scenarios that carry `tag`, in the order of the file, each with what gives it the tag. */
function carriedIn(feature: Block, tag: string): Carried[] {
  const onFeature = feature.tags.includes(tag);
  const carriers = (scenario: Block, onRule: boolean): Carrier | null => {
    if (scenario.tags.includes(tag)) {
      return "itself";
    }
    if (scenario.examples.some((examples) => examples.tags.includes(tag))) {
      return "examples";
    }
    return onRule ? "rule" : onFeature ? "feature" : null;
  };
  return feature.blocks.flatMap((block) => {
    const inside = block.kind === "rule" ? block.blocks.filter(isScenario) : isScenario(block) ? [block] : [];
    const onRule = block.kind === "rule" && block.tags.includes(tag);
    return inside.flatMap((scenario) => {
      const carrier = carriers(scenario, onRule);
      return carrier === null ? [] : [{ scenario, carrier }];
    });
  });
}

/** Whether `tag` is written anywhere in the feature: on it, a rule, a scenario or an examples block. */
function writtenIn(block: Block, tag: string): boolean {
  return block.tags.includes(tag) || block.blocks.some((inner) => writtenIn(inner, tag)) || block.examples.some((examples) => writtenIn(examples, tag));
}

function tagAddress(entry: TagEntry): string {
  return targetAddress(pageByName(entry.page));
}
