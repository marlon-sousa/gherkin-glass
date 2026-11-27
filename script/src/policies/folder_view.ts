// Policy: the view of a folder's page, the home page of a version among them.

import { targetAddress } from "./address.ts";
import { type ContentsEntry, contentsOf } from "./feature_view.ts";
import { heading, textOf } from "./heading_words.ts";
import { type LoadedFeature, type SiteFolder, folderName, folderPath, ruleCount, scenarioCount, siteFolders } from "./site_folders.ts";
import type { Block, Descriptor } from "../ports/store.ts";
import type { Parts, Words } from "../ports/words.ts";

export interface LinkView {
  readonly address: string;
  readonly text: string;
}

export interface FolderFeatureView {
  readonly address: string;
  readonly heading: Parts;
  /** The `lang` of its feature text. */
  readonly lang: string;
  readonly count: string;
  /** The accessible name of the button that shows its contents, or null when it has none, its count then plain text. */
  readonly countName: string | null;
  readonly contents: readonly ContentsEntry[];
}

export interface FolderView {
  readonly title: string;
  readonly heading: string;
  /** The site's totals, only on the home page. */
  readonly totals: readonly string[];
  readonly tagIndex: LinkView | null;
  readonly sentences: LinkView | null;
  readonly nothingGiven: string | null;
  readonly folders: { readonly heading: string; readonly links: readonly LinkView[] } | null;
  readonly features: { readonly heading: string; readonly entries: readonly FolderFeatureView[] } | null;
}

/** The page of the folder `name`, such as `billing/` or the home page's empty name, in the site of `descriptor`, whose every feature `features` holds, or null when it has none. */
export function folderView(words: Words, descriptor: Descriptor, features: readonly LoadedFeature[], name: string): FolderView | null {
  const folders = siteFolders(features);
  const folder = folders.get(name);
  if (folder === undefined) {
    return null;
  }
  const home = name === "";
  return {
    title: home ? descriptor.title : words.text("folder-page-title", { folder: folderPath(name), site: descriptor.title }),
    heading: home ? descriptor.title : folderName(name),
    totals: home ? totals(words, features, folders.size - 1) : [],
    tagIndex: home ? { address: targetAddress({ kind: "tags" }), text: words.text("tag-index-title") } : null,
    sentences: home ? { address: targetAddress({ kind: "steps" }), text: words.text("sentences-title") } : null,
    nothingGiven: home && descriptor.only && descriptor.features.length === 0 ? words.text("only-nothing-given") : null,
    folders: folder.folders.length === 0 ? null : { heading: words.text("folders"), links: folder.folders.map((inner) => ({ address: targetAddress({ kind: "folder", folder: inner }), text: folderName(inner) })) },
    features: folder.features.length === 0 ? null : { heading: words.text("features"), entries: featureEntries(words, folder) },
  };
}

function totals(words: Words, features: readonly LoadedFeature[], folders: number): string[] {
  const sum = (count: (feature: Block) => number): number => features.reduce((total, feature) => total + count(feature.object.feature), 0);
  return [
    words.text("folder-count", { count: folders }),
    words.text("feature-count", { count: features.length }),
    words.text("rule-count", { count: sum(ruleCount) }),
    words.text("scenario-count", { count: sum(scenarioCount) }),
  ];
}

function featureEntries(words: Words, folder: SiteFolder): FolderFeatureView[] {
  return folder.features.map(({ entry, object }) => {
    const featureHeading = heading(words, "feature", object.feature.name);
    const count = scenarioCount(object.feature);
    const contents = contentsOf(words, entry.page, object.feature);
    return {
      address: targetAddress({ kind: "feature", page: entry.page }),
      heading: featureHeading,
      lang: object.lang,
      count: words.text("scenario-count", { count }),
      countName: contents.length === 0 ? null : words.text("feature-scenarios", { count, feature: textOf(featureHeading) }),
      contents,
    };
  });
}
