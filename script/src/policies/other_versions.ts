// Policy: the lists of a feature's page, and of each of its blocks, in the other versions of its site, worked out from the list of versions, the index of pages and the objects other versions have for the page, as docs/BEHAVIOUR.md, rules 259 to 262, give them.

import { type PageName, targetAddress } from "./address.ts";
import { isScenario } from "./block_views.ts";
import { type Piece, pieces } from "./heading_words.ts";
import type { Block, Descriptor, PageIndex, StoreObject, VersionList } from "../ports/store.ts";
import type { Words } from "../ports/words.ts";

export interface VersionLink {
  readonly address: string;
  readonly text: string;
  /** The item's words, the link in the slot `link`, such as the link and then `: 2 blocks changed`. */
  readonly item: readonly Piece[];
}

export interface VersionGroup {
  readonly label: string;
  readonly links: readonly VersionLink[];
}

export interface OtherVersionsView {
  /** The button's words. */
  readonly label: string;
  readonly groups: readonly VersionGroup[];
}

export interface PageInVersions {
  /** Null for a page with no other version to list. */
  readonly page: OtherVersionsView | null;
  /** By the block's anchor, each scenario, scenario outline, rule and background's. */
  readonly blocks: ReadonlyMap<string, OtherVersionsView>;
}

/** A version's page, by the hash of each block counted, by its anchor, the feature's under the empty one. */
type Hashes = ReadonlyMap<string, string>;

type Comparison = { readonly kind: "missing" } | { readonly kind: "page"; readonly hashes: Hashes; readonly same: boolean };

/** The objects other versions have for `page`, other than `shown`, the object shown, that the lists need. */
export function otherObjects(index: PageIndex, page: string, shown: string): string[] {
  return (index.pages.find((entry) => entry.page === page)?.objects ?? []).map((entry) => entry.object).filter((object) => object !== shown);
}

/**
 * The lists of the page `page`, whose object `shown` is drawn, in the shell
 * of `descriptor`; `others` holds the objects `otherObjects` names.
 */
export function pageInVersions(words: Words, descriptor: Descriptor, list: VersionList, index: PageIndex, page: string, shown: { readonly name: string; readonly object: StoreObject }, others: ReadonlyMap<string, StoreObject>): PageInVersions {
  const objects = index.pages.find((entry) => entry.page === page)?.objects ?? [];
  const versions = [...list.development, ...list.releases, ...list.pre_releases].filter((version) => version !== descriptor.version);
  const shownHashes = hashesOf(shown.object.feature);
  const compared = versions.map((version): { readonly version: string; readonly comparison: Comparison } => {
    const name = objects.find((entry) => entry.versions.includes(version))?.object;
    const object = name === shown.name ? shown.object : name === undefined ? undefined : others.get(name);
    if (object === undefined) {
      return { version, comparison: { kind: "missing" } };
    }
    const hashes = hashesOf(object.feature);
    return { version, comparison: { kind: "page", hashes, same: sameHashes(shownHashes, hashes) } };
  });
  const link = (version: string, target: PageName | null, anchor: string | null): Omit<VersionLink, "item"> => ({
    address: `${descriptor.preview === null ? "../" : "../../"}${version}/index.html${target === null ? "" : targetAddress(target, anchor)}`,
    text: newer(list, version, descriptor.version) ? words.text("other-version-newer", { version }) : version,
  });
  const feature: PageName = { kind: "feature", page };
  const pageGroups = groupsOf(words, {
    changed: compared.flatMap(({ version, comparison }) => (comparison.kind === "page" && !comparison.same ? [{ ...link(version, feature, null), item: changes(words, shownHashes, comparison.hashes) }] : [])),
    missing: compared.flatMap(({ version, comparison }) => (comparison.kind === "missing" ? [{ ...link(version, null, null), item: LINK_ALONE }] : [])),
    unchanged: compared.flatMap(({ version, comparison }) => (comparison.kind === "page" && comparison.same ? [{ ...link(version, feature, null), item: LINK_ALONE }] : [])),
  });
  const blocks = new Map(
    [...shownHashes].flatMap(([anchor, hash]): [string, OtherVersionsView][] => {
      if (anchor === "") {
        return [];
      }
      const inVersion = (comparison: Comparison): "missing" | "changed" | "unchanged" => {
        const other = comparison.kind === "page" ? comparison.hashes.get(anchor) : undefined;
        return other === undefined ? "missing" : other === hash ? "unchanged" : "changed";
      };
      const blockLink = (version: string, comparison: Comparison): VersionLink => {
        const state = inVersion(comparison);
        return { ...link(version, comparison.kind === "missing" ? null : feature, state === "missing" ? null : anchor), item: LINK_ALONE };
      };
      const view = groupsOf(words, {
        changed: compared.filter(({ comparison }) => inVersion(comparison) === "changed").map(({ version, comparison }) => blockLink(version, comparison)),
        missing: compared.filter(({ comparison }) => inVersion(comparison) === "missing").map(({ version, comparison }) => blockLink(version, comparison)),
        unchanged: compared.filter(({ comparison }) => inVersion(comparison) === "unchanged").map(({ version, comparison }) => blockLink(version, comparison)),
      });
      return view === null ? [] : [[anchor, { ...view, label: words.text("other-versions-block") }]];
    }),
  );
  return { page: pageGroups && { ...pageGroups, label: words.text("other-versions-page") }, blocks };
}

function groupsOf(words: Words, links: { readonly changed: readonly VersionLink[]; readonly missing: readonly VersionLink[]; readonly unchanged: readonly VersionLink[] }): Omit<OtherVersionsView, "label"> | null {
  const groups = [
    { label: words.text("other-versions-changed"), links: links.changed },
    { label: words.text("other-versions-missing"), links: links.missing },
    { label: words.text("other-versions-unchanged"), links: links.unchanged },
  ].filter((group) => group.links.length > 0);
  return groups.length === 0 ? null : { groups };
}

/** The hash of the feature, and of each of its backgrounds, rules, scenarios and scenario outlines, by its anchor. */
function hashesOf(feature: Block): Hashes {
  const counted = (block: Block): [string, string][] =>
    block.kind === "rule" ? [[block.anchor, block.hash], ...block.blocks.flatMap(counted)] : block.kind === "background" || isScenario(block) ? [[block.anchor, block.hash]] : [];
  return new Map([["", feature.hash], ...feature.blocks.flatMap(counted)]);
}

function sameHashes(shown: Hashes, other: Hashes): boolean {
  return shown.size === other.size && [...shown].every(([anchor, hash]) => other.get(anchor) === hash);
}

const LINK_ALONE: readonly Piece[] = [{ slot: "link" }];

/** A changed page's item: its link, then its blocks changed, not there and only there, each left out when none. */
function changes(words: Words, shown: Hashes, other: Hashes): readonly Piece[] {
  const counts = [
    ["other-version-changed-blocks", [...shown].filter(([anchor, hash]) => other.has(anchor) && other.get(anchor) !== hash).length],
    ["other-version-not-there", [...shown.keys()].filter((anchor) => !other.has(anchor)).length],
    ["other-version-only-there", [...other.keys()].filter((anchor) => !shown.has(anchor)).length],
  ] as const;
  const said = counts.filter(([, count]) => count > 0).map(([id, count]) => words.text(id, { count }));
  return said.length === 0 ? LINK_ALONE : pieces(words, "other-version-changes", { changes: said.join(words.text("other-version-changes-separator")) }, ["link"]);
}

/** Whether `version` is a version number newer than `shown`'s, by the precedence the list gives. */
function newer(list: VersionList, version: string, shown: string | null): boolean {
  const rank = (name: string | null): number => list.precedence.findIndex((names) => name !== null && names.includes(name));
  const other = rank(version);
  const own = rank(shown);
  return other >= 0 && own >= 0 && other < own;
}
