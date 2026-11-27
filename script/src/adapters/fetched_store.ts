// Adapter: the store port over the site's `data/` folder, fetched over HTTP, each object kept for the rest of the visit by its name.

import { readDescriptor, readObject, readPages, readVersions } from "./fetched_store/store_format.ts";
import type { Descriptor, PageIndex, Store, StoreObject, VersionList } from "../ports/store.ts";

export class FetchedStore implements Store {
  private readonly root: URL;
  private readonly descriptorPath: string;
  private readonly fetcher: typeof fetch;
  private readonly kept = new Map<string, Promise<unknown>>();

  /** `root` is the address of `data/`, and `descriptorPath` the descriptor's below it. Chromium, through Playwright 1.63.0, refuses `fetch` called as a method of another object, so it is called on its own. */
  constructor(root: URL, descriptorPath: string, fetcher: typeof fetch = (input, init) => fetch(input, init)) {
    this.root = root;
    this.descriptorPath = descriptorPath;
    this.fetcher = fetcher;
  }

  descriptor(): Promise<Descriptor> {
    return this.file(this.descriptorPath, readDescriptor);
  }

  object(name: string): Promise<StoreObject> {
    return this.file(`objects/${encodeURIComponent(name)}.json`, readObject);
  }

  versions(): Promise<VersionList> {
    return this.file("versions.json", readVersions);
  }

  pages(): Promise<PageIndex> {
    return this.file("pages.json", readPages);
  }

  /** A file that cannot be fetched or read is forgotten, so that it is fetched again when next asked for. */
  private file<T>(path: string, read: (data: unknown) => T): Promise<T> {
    const known = this.kept.get(path) as Promise<T> | undefined;
    if (known !== undefined) {
      return known;
    }
    const fetched = this.fetched(path, read);
    this.kept.set(path, fetched);
    fetched.catch(() => this.kept.delete(path));
    return fetched;
  }

  private async fetched<T>(path: string, read: (data: unknown) => T): Promise<T> {
    const address = new URL(path, this.root);
    const response = await this.fetcher(address);
    if (!response.ok) {
      throw new Error(`${address.href}: ${response.status}`);
    }
    return read(await response.json());
  }
}
