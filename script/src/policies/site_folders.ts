// Policy: the site's folders, each with its subfolders and its features in the descriptor's order, and the counts of a feature's blocks.

import { isScenario } from "./block_views.ts";
import type { Block, FeatureEntry, StoreObject } from "../ports/store.ts";

/** A descriptor's entry, with the object it names. */
export interface LoadedFeature {
  readonly entry: FeatureEntry;
  readonly object: StoreObject;
}

export interface SiteFolder {
  /** Its page's name, such as `billing/archive/`, empty for the input folder's. */
  readonly name: string;
  readonly folders: readonly string[];
  readonly features: readonly LoadedFeature[];
}

interface FolderBeingFilled {
  readonly name: string;
  readonly folders: string[];
  readonly features: LoadedFeature[];
}

/** Every folder that has a page, by its page's name, the input folder's always among them. */
export function siteFolders(features: readonly LoadedFeature[]): ReadonlyMap<string, SiteFolder> {
  const folders = new Map<string, FolderBeingFilled>([["", { name: "", folders: [], features: [] }]]);
  const folderNamed = (name: string): FolderBeingFilled => {
    const known = folders.get(name);
    if (known !== undefined) {
      return known;
    }
    const made = { name, folders: [], features: [] };
    folders.set(name, made);
    // The descriptor lists a folder's contents together, so a subfolder is met first in its order.
    folderNamed(parentOf(name)).folders.push(name);
    return made;
  };
  for (const feature of features) {
    folderNamed(parentOf(feature.entry.page)).features.push(feature);
  }
  return folders;
}

/** The page of the folder that holds the page `name`, a feature's or a folder's. */
export function parentOf(name: string): string {
  const inner = name.endsWith("/") ? name.slice(0, -1) : name;
  const slash = inner.lastIndexOf("/");
  return slash === -1 ? "" : inner.slice(0, slash + 1);
}

/** A folder's own name, the last of its page's, such as `archive` for `billing/archive/`. */
export function folderName(name: string): string {
  return name.slice(parentOf(name).length, -1);
}

/** A folder's path below the input folder, its page's name without its last `/`. */
export function folderPath(name: string): string {
  return name.slice(0, -1);
}

/** Its scenarios, outlines counted once, those inside rules among them. */
export function scenarioCount(feature: Block): number {
  return feature.blocks.reduce((count, block) => count + (isScenario(block) ? 1 : block.kind === "rule" ? block.blocks.filter(isScenario).length : 0), 0);
}

export function ruleCount(feature: Block): number {
  return feature.blocks.filter((block) => block.kind === "rule").length;
}
