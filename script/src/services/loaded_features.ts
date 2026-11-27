// Service: every feature a descriptor names, with its object fetched from the store.

import type { LoadedFeature } from "../policies/site_folders.ts";
import type { Descriptor, Store } from "../ports/store.ts";

/** Rejects when an object cannot be fetched, or is not one. */
export function loadedFeatures(store: Store, descriptor: Descriptor): Promise<LoadedFeature[]> {
  return Promise.all(descriptor.features.map(async (entry) => ({ entry, object: await store.object(entry.object) })));
}
