// Service: every feature a descriptor names, with its object fetched from the store, and the documents search reads, made from them.

import { searchDocuments } from "../policies/search_documents.ts";
import type { LoadedFeature } from "../policies/site_folders.ts";
import type { Documents } from "../ports/documents.ts";
import type { Descriptor, Store } from "../ports/store.ts";

/** Rejects when an object cannot be fetched, or is not one. */
export function loadedFeatures(store: Store, descriptor: Descriptor): Promise<LoadedFeature[]> {
  return Promise.all(descriptor.features.map(async (entry) => ({ entry, object: await store.object(entry.object) })));
}

/** The documents of the store's descriptor, made each time they are asked for, from the objects the store keeps. */
export function storeDocuments(store: Store): Documents {
  return { load: async () => searchDocuments(await loadedFeatures(store, await store.descriptor())) };
}
